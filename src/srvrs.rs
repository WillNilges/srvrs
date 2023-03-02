// To watch directories
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::{path::PathBuf, fs};
use file_owner::PathExt;
use chrono;

// To run commands based on said directories
use std::process::Command;

pub struct Srvrs {
    pub primary_path: String,
    pub work_path: String,
    pub command: String,
}

impl Srvrs {
    pub fn launch(&self) {
        println!("watching {}", self.primary_path);
        if let Err(e) = self.watch() {
            println!("error: {:?}", e)
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
                    //println!("changed: {:?}", event);
                    match event.kind {
                        notify::EventKind::Create(notify::event::CreateKind::File) => {
                            println!("changed: {:?}", event);
                            // TODO: Make this app work with multiple paths at once
                            //println!("{:?}", event.paths); // Debug for seeing event info
                            self.respond(event.paths);
                        },
                        _ => {
                        }
                    }
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
        Ok(())
    }

    fn respond(&self, files: Vec<PathBuf>) {
        // Pick the first file created out of there.
        let first_file = files[0].display().to_string();
        let first_file_name = files[0].file_name().unwrap().to_string_lossy();
        let first_file_name_prefix = files[0].file_stem().unwrap().to_string_lossy(); // TODO: Wait for
        // prefix to get out of nightly so we can use that instead of file_stem()
        
        // Get the owner of the path so we can put our output in their homedir.
        let o = first_file.owner().unwrap();
        let owner = o.name().unwrap().unwrap();
        
        println!("{} Just uploaded a file at {}!", owner, first_file);

        // TODO: Check if it's a video/audio file
        
        // TODO: Make a temp directory in our own output directory
        // MOVE the file over to there, then launch the command on
        // that path.
        // workpath=/var/srvrs/work/<FILE_NAME_AND_OWNER>
        // mkdir $workpath
        // mv $first_file $workpath
        
        // Create temp work directory. We'll put the file here, then run the command we
        // were given on it.
        let new_user_work_dir = format!("{}/{}_{}", self.work_path, owner, first_file_name_prefix);
        println!("Creating {} for new user work.", new_user_work_dir);
        fs::create_dir(&new_user_work_dir)
            .unwrap_or_else(|e| panic!("Error creating dir: {}", e));

        // Move file into temp work directory
        let new_user_file_path = format!("{}/{}", new_user_work_dir, first_file_name);
        fs::rename(first_file, &new_user_file_path)
            .unwrap_or_else(|e| panic!("Error copying file: {}", e));

        println!("Running command!");

        let built_command = self.command.to_owned() + &new_user_file_path;
        let output = Command::new("sh")
                    .arg("-c")
                    .arg(built_command)
                    .output()
                    .expect("failed to execute process");

        let hello = output.stdout;
        println!("{}", String::from_utf8_lossy(&hello));

        // When finished, move the work directory into the user's scratchdir.
        // TODO: Create it if it doesn't exist.
        println!("Moving results to scratch!");
        let home = "/scratch";
        fs::rename(new_user_work_dir, format!("{}/{}/{}_{}_{}", home, owner, "srvrs", chrono::offset::Local::now().timestamp(), first_file_name_prefix))
            .unwrap_or_else(|e| panic!("Error copying file: {}", e));
    }
}
