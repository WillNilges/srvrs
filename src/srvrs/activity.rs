use anyhow::{anyhow, Result};
use file_owner::PathExt;
use log::{error, info, warn, LevelFilter};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use simple_logger::SimpleLogger;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use serde::{de, Deserialize};

// An activity is, simply put, a "thing that SRVRS can do for you."
#[derive(Deserialize, Debug)]
pub struct Activity {
    pub name: String, // The name of this activity 
    pub script: String, // Path to script this will run 
    #[serde(deserialize_with = "wants_deserializer")]
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
                _ => Err(de::Error::invalid_value(de::Unexpected::Str(t), &"MatcherType")),
            }
        )
        .collect()
}

impl Activity {
    pub fn launch(&self) {
        // TODO: Parse script and make sure it's formatted correctly?
        SimpleLogger::new().init().unwrap();
        log::set_max_level(LevelFilter::Info);
        info!(
            "Watching {}. Will run `{}` when a file is added.",
            self.name, self.script
        );
        self.update_status(
            format!("Idle. Upload a file to {} to get started.", self.watch_dir)
        );
        if let Err(e) = self.watch() {
            error!("error: {:?}", e)
        }
    }

    fn update_status(&self, status: String) {
        fn write_status(path: &str, status: String) -> Result<()> {
            let mut sf = fs::File::create(path)?;
            sf.write_all(status.as_bytes())?;
            Ok(())
        }

        write_status(&self.status_path, status)
            .unwrap_or_else(|_|error!("Could not update status"));
    }

    fn update_queue(&self) {
        fn write_queue(watch_dir: &str, queue_path: &str) -> Result<()> {
            let paths = fs::read_dir(watch_dir).unwrap();

            let mut qf = fs::File::create(queue_path)?;
            let mut queue_files = String::from("");
            for path in paths {
                queue_files.push_str(
                    format!("{}\n", path.unwrap().path().display()).as_str()
                );
            }
            qf.write_all(queue_files.as_bytes())?;
            Ok(())
        }

        write_queue(&self.watch_dir, &self.queue_path)
            .unwrap_or_else(|_| error!("Could not update queue"));
    }

    fn run_script(&self, input: String) -> Result<()> {
        let script = &self.script;
        let mut cmd = Command::new(script)
            .arg(&input)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            match line {
                Ok(l) => {
                    info!("{}", l);
                    let re = Regex::new(&self.progress_regex).unwrap();
                    for caps in re.captures_iter(&l) {
                        self.update_status(
                            format!(
                                "Running {}...\n{}", 
                                self.name, caps.get(1).unwrap().as_str()
                            )
                        );
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
                                            format!(
                                                "Idle. Upload a file to {} to get started.",
                                                self.watch_dir
                                            )
                                        );
                                    }
                                Err(e) => {
                                    error!("Error responding to file: {}", e);
                                    let condemned_path: String =
                                        event.paths[0].to_string_lossy().to_string();
                                    warn!("Deleting {}", &condemned_path);
                                    fs::remove_file(condemned_path)?;
                                    self.update_status(
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

        // Check if it's the right kind of file
        let kind = match infer::get_from_path(&file) {
            Ok(file_read) => match file_read {
                Some(file_type) => file_type,
                _ => return Err(anyhow!("Could not infer type of {}", file)),
            },
            _ => return Err(anyhow!("Could not infer type of {}", file)),
        };
        if !self.wants.contains(&kind.matcher_type()) {
            return Err(anyhow!(
                "{} is an unsupported file type. Found {}?",
                &file,
                kind.mime_type()
            ))
        }
        info!("{} is a {:?}", &file, kind.matcher_type());
        
        // Create temp work directory. We'll put the file here, then run the command we
        // were given on it.
        let file_work_dir = format!("{}/{}_{}", self.work_dir, owner, file_prefix);
        info!("Creating {} for new user work.", file_work_dir);
        fs::create_dir(&file_work_dir)?;

        // Move file into temp work directory
        let file_work_path = format!("{}/{}", file_work_dir, file_name);
        fs::rename(file, &file_work_path)?;

        info!("Running command: {}", self.script.to_owned());
        self.update_status("Launching command...".to_string());

        self.run_script(file_work_path)?;

        // When finished, move the work directory into the distributor directory
        // so that the distributor can send it to the user.
        info!("Moving to distributor");
        self.update_status("Moving to distributor...".to_string());
        fs::rename(file_work_dir, format!("{}/{}", self.distributor_dir, owner))?;

        Ok(())
    }
}
