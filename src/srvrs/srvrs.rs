// To watch directories
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::{path::PathBuf, fs};
use file_owner::PathExt;
use anyhow::{anyhow, Result};
use log::{info, error, LevelFilter};
use simple_logger::SimpleLogger;
use std::process::{Command, Stdio};
use std::path::Path;
use std::io::{BufReader, BufRead};

pub fn exec_stream<P: AsRef<Path>>(binary: P, args: Vec<String>) {
    let mut cmd = Command::new(binary.as_ref())
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdout = cmd.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines {
            info!("Read: {:?}", line);
        }
    }

    cmd.wait().unwrap();
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

        // Add a path to be watched. All files and directories at that path
        // will be watched for changes.
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
                            match self.respond(event.paths) {Ok(()) => {}, Err(e) => println!("Error responding to file: {}",e),};
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
        // Pick the first file created.
        let file = files[0].display().to_string();
        let file_name = match files[0].file_name() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("Invalid file name")),
        };

        let file_stem = match files[0].file_stem() {
            Some(name) => name.to_string_lossy(),
            None => return Err(anyhow!("Invalid file stem")),
        }; // TODO: Wait for
        // prefix to get out of nightly so we can use that instead of file_stem()
        
        // Get the owner of the path so we can put our output in their homedir.
        let owner = match file.owner()?.name()? {
            Some(name) => name,
            None => return Err(anyhow!("Could not find an owner for this file!")),
        };
        
        info!("{} Just uploaded a file at {}!", owner, file);

        // TODO: Check if it's a video/audio file
        
        // Make a temp directory in our own output directory
        // MOVE the file over to there, then launch the command on
        // that path.
        // workpath=/var/srvrs/work/<FILE_NAME_AND_OWNER>
        // mkdir $workpath
        // mv $file $workpath
        
        // Create temp work directory. We'll put the file here, then run the command we
        // were given on it.
        let work_dir = format!("{}/{}_{}", self.work_path, owner, file_stem);
        info!("Creating {} for new user work.", work_dir);
        fs::create_dir(&work_dir)?;

        // Move file into temp work directory
        let work_path = format!("{}/{}", work_dir, file_name);
        fs::rename(file, &work_path)?;

        info!("Running command: {} -p {}", self.command.to_owned(), &work_path);

        exec_stream(self.command.to_owned(), vec!("-p".to_string(), work_path));

        // When finished, move the work directory into the distributor directory so that the
        // distributor can send it to the user. 
        // TODO: Create user's scratch directory if it doesn't exist.
        info!("Moving to distributor!");
        fs::rename(work_dir, format!("{}/{}", self.distributor_path, owner))?;

        /*
        info!("Moving results to {}!", self.destination_base_path);
        fs::rename(work_dir, format!("{}/{}/{}_{}_{}", self.destination_base_path, owner, "srvrs", chrono::offset::Local::now().timestamp(), file_stem))?;
        */
        Ok(())
    }
}
