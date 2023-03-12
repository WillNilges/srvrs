use std::{os::unix::fs::chown, fs::rename};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use chrono;
use log::{info, error, LevelFilter};
use users::{get_user_by_name, get_group_by_name};

pub struct Distributor {
    pub work_path: String,
    pub destination_base_path: String,
}

impl Distributor {
    pub fn launch(&self) {
        systemd_journal_logger::init().unwrap();
        log::set_max_level(LevelFilter::Info);
        info!("Distributor Active.");
        info!("Watching {}. Will move files when file is added.", self.work_path);
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
                    match event.kind {
                        // Only take action after the file is finished writing.
                        notify::EventKind::Create(
                            notify::event::CreateKind::Folder
                        ) => {
                            info!("Got new file: {:?}", event);
                            let file = event.paths[0].display().to_string();
                            let file_name = event.paths[0].file_name().unwrap().to_string_lossy();

                            // This app will watch /var/srvrs/distributor, which the main app will
                            // move finished work to in one shot.

                            // When srvrs is finished, move the work directory into the user's scratchdir.
                            info!("Moving {}'s results to {}", file_name, self.destination_base_path);
                
                            // Create user's scratch directory if it doesn't exist
                            std::fs::create_dir_all(format!(
                                "{}/{}",
                                self.destination_base_path,
                                file_name,
                            ))?;

                            let file_dest = format!(
                                "{}/{}/{}_{}",
                                self.destination_base_path,
                                file_name,
                                "srvrs",
                                chrono::offset::Local::now().timestamp()
                            );

                            // Move the file
                            rename(
                                file,
                                &file_dest
                            )?;

                            // Change ownership of file
                            // FIXME: This doesn't change all ownership!
                            let owner = file_name.to_string();
                            let my_uid: u32 = match get_user_by_name(&owner) {
                                    Some(user) => user.uid(),
                                    _ => panic!("User not found >:("),
                                };
                            let my_gid: u32 = match get_group_by_name("member") {
                                    Some(group) => group.gid(),
                                    _ => panic!("Group not found >:("),
                                };

                            info!("UID: {}, GID: {}", my_uid, my_gid);
                            chown(file_dest, Some(my_uid), Some(my_gid))?;
                        },
                        _ => {}
                    }
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
        Ok(())
    }
}
