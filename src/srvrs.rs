// To watch directories
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::{path::PathBuf, fs};
use file_owner::PathExt;
use chrono;
use anyhow::{anyhow, Result};
use log::{info, error, LevelFilter};

// To run commands based on said directories
use std::process::Command;

pub struct Srvrs {
    pub primary_path: String,
    pub work_path: String,
    pub destination_base_path: String,
    pub command: String,
}

impl Srvrs {
    pub fn launch(&self) {
        systemd_journal_logger::init().unwrap();
        log::set_max_level(LevelFilter::Info);
        info!("Watching {}. Will run `{}` when a file is added.", self.primary_path, self.command);
        if let Err(e) = self.watch() {
            error!("error: {:?}", e)
        }  
    }

    fn watch(&self) -> notify::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(self.primary_path.as_ref(), RecursiveMode::NonRecursive)?;

        for res in rx {
            match res {
                Ok(event) => {
                    match event.kind {
                        // Only take action after the file is finished writing.
                        notify::EventKind::Access(
                            notify::event::AccessKind::Close(
                                notify::event::AccessMode::Write
                            )
                        ) => {
                            info!("changed: {:?}", event);

                            // TODO: Make this app work with multiple paths at once
                            match self.respond(event.paths) {Ok(()) => {}, Err(e) => println!("{}",e),};
                        },
                        _ => {
                        }
                    }
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
        Ok(())
    }

    fn respond(&self, files: Vec<PathBuf>) -> Result<()> {
        // Pick the first file created out of there.
        let first_file = files[0].display().to_string();
        let first_file_name = match files[0].file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("Invalid file name")),
        };

        let first_file_name_prefix = match files[0].file_stem() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("Invalid file stem")),
        }; // TODO: Wait for
        // prefix to get out of nightly so we can use that instead of file_stem()
        
        // Get the owner of the path so we can put our output in their homedir.
        let owner = match first_file.owner()?.name()? {
            Some(name) => name,
            None => return Err(anyhow!("Could not find an owner for this file!")),
        };
        
        info!("{} Just uploaded a file at {}!", owner, first_file);

        // TODO: Check if it's a video/audio file
        
        // Make a temp directory in our own output directory
        // MOVE the file over to there, then launch the command on
        // that path.
        // workpath=/var/srvrs/work/<FILE_NAME_AND_OWNER>
        // mkdir $workpath
        // mv $first_file $workpath
        
        // Create temp work directory. We'll put the file here, then run the command we
        // were given on it.
        let new_user_work_dir = format!("{}/{}_{}", self.work_path, owner, first_file_name_prefix);
        info!("Creating {} for new user work.", new_user_work_dir);
        fs::create_dir(&new_user_work_dir)?;

        // Move file into temp work directory
        let new_user_file_path = format!("{}/{}", new_user_work_dir, first_file_name);
        fs::rename(first_file, &new_user_file_path)?;

        let built_command = format!("{} {}", self.command.to_owned(), &new_user_file_path);
        info!("Running command: {}", built_command);
        let output = Command::new("sh")
                    .arg("-c")
                    .arg(built_command)
                    .output()?;

        let hello = output.stdout;
        info!("{}", String::from_utf8_lossy(&hello));

        // When finished, move the work directory into the user's scratchdir.
        // TODO: Create it if it doesn't exist.
        info!("Moving results to {}!", self.destination_base_path);
        fs::rename(new_user_work_dir, format!("{}/{}/{}_{}_{}", self.destination_base_path, owner, "srvrs", chrono::offset::Local::now().timestamp(), first_file_name_prefix))?;
        Ok(())
    }
}
