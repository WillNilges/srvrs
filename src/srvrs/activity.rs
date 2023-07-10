use anyhow::{anyhow, Result};
use file_owner::PathExt;
use log::{error, info, warn};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
    collections::HashMap,
    os::unix::fs::{chown, PermissionsExt},
};
use serde::{de, Deserialize};
use crate::{SRVRS_UID, SRVRS_GID, MEMBERS_GID};
use crate::gpu::wait_for_gpu;

#[derive(Deserialize, Debug)]
pub struct SrvrsConfig {
    pub base_dir: String,
    pub activities: HashMap<String, ActivityConfig>
}

// An activity is, simply put, a "thing that SRVRS can do for you."
#[derive(Deserialize, Debug)]
pub struct ActivityConfig {
    //pub name: String, // The name of this activity 
    pub script: String, // Path to script this will run 
    #[serde(deserialize_with = "wants_deserializer")]
    pub wants: Vec<infer::MatcherType>, // The kinds of file the script accepts
    pub progress_regex: String, // Regex for caputring status from output
}

pub struct Activity {
    pub name: String, // The name of this activity 
    pub script: String, // Path to script this will run 
    pub wants: Vec<infer::MatcherType>, // The kinds of file the script accepts
    pub progress_regex: String, // Regex for caputring status from output
    pub watch_dir: String, // The dir this Activity will watch for work
    pub status_path: String, // The file this Activity will report status 
    pub queue_path: String, // The file this Activity will report queue
    pub work_dir: String, // The dir work is done
    pub distributor_dir: String, // The dir to put finished work in
}

fn wants_deserializer<'de, D>(deserializer: D) -> Result<Vec<infer::MatcherType>, D::Error>
where
  D: de::Deserializer<'de>,
{

    let string_types: Vec<&str> = Deserialize::deserialize(deserializer)?;
    string_types
        .into_iter()
        .map(|t|
            match t {
                "App" => Ok(infer::MatcherType::App),
                "Archive" => Ok(infer::MatcherType::Archive),
                "Audio" => Ok(infer::MatcherType::Audio),
                "Book" => Ok(infer::MatcherType::Book),
                "Doc" => Ok(infer::MatcherType::Doc),
                "Font" => Ok(infer::MatcherType::Font),
                "Image" => Ok(infer::MatcherType::Image),
                "Text" => Ok(infer::MatcherType::Text),
                "Video" => Ok(infer::MatcherType::Video),
                "Custom" => Ok(infer::MatcherType::Custom),
                "Any" => Ok(infer::MatcherType::Custom),
                _ => Err(de::Error::invalid_value(de::Unexpected::Str(t), &"MatcherType")),
            }
        )
        .collect()
}

#[derive(Debug)]
enum StatusSummary {
    IDLE,
    STARTING,
    RUNNING,
    CLEANUP,
    ERROR,
}

impl Activity {

    // Setup the service and watch the requisite directories
    pub async fn launch(&self) {
        // TODO: Parse script and make sure it's formatted correctly?
        info!(
            "Watching {}. Will run `{}` when a file is added.",
            self.name, self.script
        );
        self.update_status(
            StatusSummary::IDLE,
            "".to_string()
        );
        if let Err(e) = self.watch() {
            error!("error: {:?}", e);
        } 
    }

    // Write a string to a file, presumably, the string is output from a script.
    fn update_status(&self, summary: StatusSummary, status: String) {
        fn write_status(path: &str, name: &str, summary: StatusSummary, status: String) -> Result<()> {
            let mut sf = fs::File::create(path)?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o644))?;
            chown(path, Some(*SRVRS_UID), Some(*MEMBERS_GID))?;
            let status_update: String = format!(
                "{} - {:#?}:\n{}",
                name,
                summary,
                status
            );
            sf.write_all(status_update.as_bytes())?;
            Ok(())
        }

        write_status(&self.status_path, &self.name, summary, status)
            .unwrap_or_else(|_|error!("Could not update status"));

        self.update_queue();
    }

    fn update_queue(&self) {
        fn write_queue(name: &str, watch_dir: &str, queue_path: &str) -> Result<()> {
            let paths = fs::read_dir(watch_dir).unwrap();

            let mut qf = fs::File::create(queue_path)?;
            fs::set_permissions(queue_path, fs::Permissions::from_mode(0o644))?;
            chown(queue_path, Some(*SRVRS_UID), Some(*MEMBERS_GID))?;
            let mut queue_files = String::from(format!("{}:\n", name));
            for path in paths {
                queue_files.push_str(
                    format!("{}\n", path.unwrap().path().display()).as_str()
                );
            }
            qf.write_all(queue_files.as_bytes())?;
            Ok(())
        }

        write_queue(&self.name, &self.watch_dir, &self.queue_path)
            .unwrap_or_else(|_| error!("Could not update queue"));
    }

    // Run whatever script is attached to the activity and use a regex to try
    // capturing status updates
    fn run_script(&self, input: String, gpus: String) -> Result<()> {
        let script = &self.script;
        let mut cmd = Command::new(script)
            .arg(&input)
            .arg(&gpus)
            .stdout(Stdio::piped())
            .spawn()?;

        let cmd_stdout = cmd.stdout.as_mut().unwrap();
        let cmd_stdout_reader = BufReader::new(cmd_stdout);
        let cmd_stdout_lines = cmd_stdout_reader.lines();

        for line in cmd_stdout_lines {
            match line {
                Ok(l) => {
                    info!("{}", l);
                    let sus_re = Regex::new(&self.progress_regex);
                    match sus_re {
                        Ok(re)  => {
                            for caps in re.captures_iter(&l) {
                                //info!("Regex Matched: {}", l);
                                // https://docs.rs/regex/latest/regex/struct.Regex.html#method.captures
                                self.update_status(
                                    StatusSummary::RUNNING,
                                    caps.get(0).unwrap().as_str().to_string()
                                );
                            }
                        },
                        Err(bad_re) => {
                            warn!("Got bad regex: {}", bad_re);
                        },
                    }
                }
                _ => warn!("Could not read command ouput."),
            };
        }

        cmd.wait().unwrap();
        Ok(())
    }

    fn watch(&self) -> notify::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path
        // will be watched for changes.
        watcher.watch(self.watch_dir.as_ref(), RecursiveMode::NonRecursive)?;

        for res in rx {
            match res {
                Ok(event) => {
                    // Update number of files in the queue
                    self.update_queue();
                    match event.kind {
                        // Only take action after the file is finished writing.
                        notify::EventKind::Access(notify::event::AccessKind::Close(
                            notify::event::AccessMode::Write,
                        )) => {
                            info!("changed: {:?}", event);
                            match self.respond(&event.paths) {
                                Ok(()) => {
                                        self.update_status(
                                            StatusSummary::IDLE,
                                            "".to_string()
                                        );
                                    }
                                Err(e) => {
                                    error!("Error responding to file: {}", e);
                                    let condemned_path: String =
                                        event.paths[0].to_string_lossy().to_string();
                                    warn!("Deleting {}", &condemned_path);
                                    fs::remove_file(condemned_path)?;
                                    self.update_status(
                                        StatusSummary::ERROR,
                                        format!("Error responding to file: {}", e)
                                    );
                                }
                            };
                        }
                        _ => {}
                    }
                }
                Err(e) => error!("watch error: {:?}", e),
            }
        }
        Ok(())
    }

    fn respond(&self, files: &Vec<PathBuf>) -> Result<()> {
        // Pick the first file created.
        let file = files[0].display().to_string();
        let file_name = match files[0].file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("Invalid file name")),
        };

        let file_prefix = match files[0].file_prefix() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("Invalid file prefix")),
        };

        // Get the owner of the path so we can put our output in their homedir.
        let owner = match file.owner()?.name()? {
            Some(name) => name,
            None => return Err(anyhow!("Could not find an owner for this file")),
        };

        info!("{} uploaded {}", owner, file);

        // Hacky skip to get stable diffusion working with raw text files.
        if !(self.wants[0] == infer::MatcherType::Text && self.wants.len() == 1) {
            // Check if it's the right kind of file
            let kind = match infer::get_from_path(&file) {
                Ok(file_read) => match file_read {
                    Some(file_type) => file_type,
                    _ => return Err(anyhow!("Could not infer type of {}", file)),
                },
                _ => return Err(anyhow!("Could not find file: {}", file)),
            };
            if !self.wants.contains(&kind.matcher_type()) {
                return Err(anyhow!(
                    "{} is an unsupported file type. Found {}?",
                    &file,
                    kind.mime_type()
                ))
            }
            info!("{} is a {:?}", &file, kind.matcher_type());
        }

        // Wait for a GPU to be free
        // TODO: The '1' is a placeholder for the field passed in through the command line
        let gpus = wait_for_gpu(1)?;
        
        // Create temp work directory. We'll put the file here, then run the command we
        // were given on it.
        let file_work_dir = format!("{}/{}_{}", self.work_dir, owner, file_prefix);
        info!("Creating {} for new user work.", file_work_dir);
        fs::create_dir(&file_work_dir)?;

        // Move file into temp work directory
        let file_work_path = format!("{}/{}", file_work_dir, file_name);
        fs::rename(file, &file_work_path)?;

        info!("Running command: {}", self.script.to_owned());
        self.update_status(
            StatusSummary::STARTING,
            "Launching command...".to_string()
        );

        self.run_script(file_work_path, gpus)?;

        // When finished, move the work directory into the distributor directory
        // so that the distributor can send it to the user.
        info!("Moving to distributor");
        self.update_status(
            StatusSummary::CLEANUP,
            "Moving to distributor...".to_string()
        );
        fs::rename(file_work_dir, format!("{}/{}", self.distributor_dir, owner))?;

        Ok(())
    }
}
