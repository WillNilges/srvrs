// To watch directories
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::fs;
use chrono;
use log::{info, error, LevelFilter};

pub struct Distributor {
    pub work_path: String,
    pub destination_base_path: String,
}

impl Distributor {
    pub fn launch(&self) {
        //systemd_journal_logger::init().unwrap();
        log::set_max_level(LevelFilter::Info);
        println!("Distributor Active.");
        println!("Watching {}. Will move files when file is added.", self.work_path);
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
        watcher.watch(self.work_path.as_ref(), RecursiveMode::Recursive)?;

        for res in rx {
            match res {
                Ok(event) => {
                    println!("Got new file: {:?}", event);
                    match event.kind {
                        // Only take action after the file is finished writing.
                        notify::EventKind::Create(
                            notify::event::CreateKind::Folder
                        ) => {

                            println!("Got new file: {:?}", event);
                            // Pick the first file created out of there.
                            let first_file = event.paths[0].display().to_string();
                            let first_file_name = event.paths[0].file_name().unwrap().to_string_lossy();

                            // This app will watch /var/srvrs/distributor, which the main app will
                            // move finished work to in one shot.
                            println!("Moving it now.");


                            // TODO: Directory name will be username. All this app needs to do
                            // is move everything in that directory to that user's scratch
                            // directory. 
 
                            // When srvrs is finished, move the work directory into the user's scratchdir.
                            // TODO: Create it if it doesn't exist.
                            println!("Moving {}'s results to {}!", first_file_name, self.destination_base_path);
                            fs::rename(
                                first_file,
                                format!(
                                    "{}/{}/{}_{}",
                                    self.destination_base_path,
                                    first_file_name,
                                    "srvrs",
                                    chrono::offset::Local::now().timestamp()
                                )
                            )?;
                           
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
}
