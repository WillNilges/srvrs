use anyhow::{anyhow, Result};
use file_owner::PathExt;
use log::{error, info, warn, LevelFilter};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use simple_logger::SimpleLogger;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn update_queue() -> Result<()> {
    let paths = fs::read_dir("/var/srvrs/whisper").unwrap();

    let mut qf = fs::File::create("/var/srvrs/queue")?;
    let mut queue_files = String::from("");
    for path in paths {
        queue_files.push_str(format!("{}\n", path.unwrap().path().display()).as_str());
    }
    qf.write_all(queue_files.as_bytes())?;
    Ok(())
}

fn update_status(status: String) -> Result<()> {
    let mut sf = fs::File::create("/var/srvrs/status")?;
    sf.write_all(status.as_bytes())?;
    Ok(())
}

fn exec_stream<P: AsRef<Path>>(binary: P, args: Vec<String>) -> Result<()> {
    let mut cmd = Command::new(binary.as_ref())
        .args(&args)
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
                // [0-9][0-9]:[0-9][0-9].[0-9][0-9][0-9](?= -->)
                let re = Regex::new(r"([0-9][0-9]:[0-9][0-9].[0-9][0-9][0-9])( -->)").unwrap();
                for caps in re.captures_iter(&l) {
                    update_status(format!("Running whisper...\n{}", caps.get(1).unwrap().as_str()))?;
                }
            }
            _ => warn!("Could not read command ouput."),
        };
    }

    cmd.wait().unwrap();
    Ok(())
}

pub struct Srvrs {
    pub primary_path: String,
    pub work_path: String,
    pub command: String,
    pub distributor_path: String,
}

impl Srvrs {
    pub fn launch(&self) {
        SimpleLogger::new().init().unwrap();
        //systemd_journal_logger::init().unwrap();
        log::set_max_level(LevelFilter::Info);
        info!(
            "Watching {}. Will run `{}` when a file is added.",
            self.primary_path, self.command
        );
        update_status("Idle. Upload a file to /var/srvrs/whisper to get started.".to_string()).unwrap_or_else(|_| error!("Could not update status"));
        if let Err(e) = self.watch() {
            error!("error: {:?}", e)
        }
    }

    fn watch(&self) -> notify::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path
        // will be watched for changes.
        watcher.watch(self.primary_path.as_ref(), RecursiveMode::NonRecursive)?;

        for res in rx {
            match res {
                Ok(event) => {
                    // Update number of files in the queue
                    update_queue().unwrap_or_else(|_| error!("Could not update queue"));
                    match event.kind {
                        // Only take action after the file is finished writing.
                        notify::EventKind::Access(notify::event::AccessKind::Close(
                            notify::event::AccessMode::Write,
                        )) => {
                            info!("changed: {:?}", event);
                            match self.respond(&event.paths) {
                                Ok(()) => {
                                        update_status("Idle. Upload a file to /var/srvrs/whisper to get started.".to_string()).unwrap_or_else(|_| error!("Could not update status"));
                                    }
                                Err(e) => {
                                    error!("Error responding to file: {}", e);
                                    let condemned_path: String =
                                        event.paths[0].to_string_lossy().to_string();
                                    warn!("Deleting {}", &condemned_path);
                                    fs::remove_file(condemned_path)?;
                                    update_status(format!("Error responding to file: {}", e)).unwrap_or_else(|_| error!("Could not update status"));
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

        // TODO: Check if it's a video/audio file
        let kind = match infer::get_from_path(&file) {
            Ok(file_read) => match file_read {
                Some(file_type) => file_type,
                _ => return Err(anyhow!("Could not infer type of {}", file)),
            },
            _ => return Err(anyhow!("Could not infer type of {}", file)),
        };

        match kind.matcher_type() {
            infer::MatcherType::Audio => info!("{} is an audio file.", &file),
            infer::MatcherType::Video => info!("{} is a video file.", &file),
            _ => {
                return Err(anyhow!(
                    "{} is an unsupported file type. Found {}?",
                    &file,
                    kind.mime_type()
                ))
            }
        }

        // Create temp work directory. We'll put the file here, then run the command we
        // were given on it.
        let work_dir = format!("{}/{}_{}", self.work_path, owner, file_prefix);
        info!("Creating {} for new user work.", work_dir);
        fs::create_dir(&work_dir)?;

        // Move file into temp work directory
        let work_path = format!("{}/{}", work_dir, file_name);
        fs::rename(file, &work_path)?;

        info!("Running command: {}", self.command.to_owned());
        update_status("Launching command...".to_string())?;

        exec_stream(self.command.to_owned(), vec!["-p".to_string(), work_path])?;

        // When finished, move the work directory into the distributor directory
        // so that the distributor can send it to the user.
        info!("Moving to distributor");
        update_status("Moving to distributor...".to_string())?;
        fs::rename(work_dir, format!("{}/{}", self.distributor_path, owner))?;

        Ok(())
    }
}
