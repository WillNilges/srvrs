#![feature(path_file_prefix)]
#![feature(unix_chown)]
use clap::{Args, Parser, Subcommand};
use std::{io::Read, fs, os::unix::fs::{PermissionsExt, chown}};
use serde_yaml;
use tokio;
use log::{error, info, LevelFilter};
use simple_logger::SimpleLogger;
use users::{get_user_by_name, get_group_by_name};
use lazy_static::lazy_static;
use anyhow::Error;

pub mod activity;
pub mod gpu;

lazy_static! {
    static ref MEMBERS_GID: u32 = match get_group_by_name("member") {
        Some(group) => group.gid(),
        _ => panic!("Group not found >:("),
    };

    static ref SRVRS_UID: u32 = match get_user_by_name("srvrs") {
        Some(user) => user.uid(),
        _ => panic!("User not found"),
    };

    static ref SRVRS_GID: u32 = match get_group_by_name("srvrs") {
        Some(group) => group.gid(),
        _ => panic!("Group not found"),
    };
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct SubCommands {
    #[command(subcommand)]
    subcommand: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    Setup(WatchArgs),
    Watch(WatchArgs),
    Status,
    Services,
    Queue,
}

#[derive(Args, Debug)]
struct WatchArgs {
    /// Config file 
    #[arg(short, long, required = true)]
    config_file: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = SubCommands::parse();
    SimpleLogger::new().init().unwrap();
    log::set_max_level(LevelFilter::Debug); // TODO: Configure this somehow?
            
    match args.subcommand {
        Action::Setup(watch_args) => {
            println!("Running setup...");
            let config = fs::read_to_string(watch_args.config_file).unwrap();
            let sc: activity::SrvrsConfig = serde_yaml::from_str(&config).unwrap();

            // All the required directories
            //let watch_dir = format!("{}/watch", sc.base_dir);
            let scripts_dir = format!("{}/scripts", sc.base_dir);
            let status_dir = format!("{}/status", sc.base_dir);
            let queue_dir = format!("{}/queue", sc.base_dir);
            let work_dir = format!("{}/work", sc.base_dir);
            let distributor_dir = format!("{}/distributor", sc.base_dir);

            // Create base directories for srvrs
            for dir in vec![&scripts_dir, &work_dir, &distributor_dir] {
                info!("Creating directory: {}", &dir);
                fs::create_dir_all(&dir).unwrap();
                fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
                chown(dir, Some(*SRVRS_UID), Some(*SRVRS_GID)).unwrap();
            }

            // Create status and queue directories
            for dir in vec![&status_dir, &queue_dir] {
                info!("Creating directory: {}", &dir);
                fs::create_dir_all(&dir).unwrap();
                fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
                chown(dir, Some(*SRVRS_UID), Some(*MEMBERS_GID)).unwrap();
            }

            // Create work directory for each activity
            for (name, _) in &sc.activities {
                let activity_dir = format!("{}/{}", sc.base_dir, name);
                info!("Creating directory: {}", activity_dir);
                fs::create_dir_all(&activity_dir).unwrap();
                fs::set_permissions(&activity_dir, fs::Permissions::from_mode(0o730)).unwrap();

                chown(activity_dir, Some(*SRVRS_UID), Some(*MEMBERS_GID)).unwrap();
            }

            println!("Finished!");
        },
        Action::Watch(watch_args) => {
            let config = fs::read_to_string(watch_args.config_file).unwrap();
            let sc: activity::SrvrsConfig = serde_yaml::from_str(&config).unwrap();

            // All the required directories
            //let watch_dir = format!("{}/watch", sc.base_dir);
            let scripts_dir = format!("{}/scripts", sc.base_dir);
            let status_dir = format!("{}/status", sc.base_dir);
            let queue_dir = format!("{}/queue", sc.base_dir);
            let work_dir = format!("{}/work", sc.base_dir);
            let distributor_dir = format!("{}/distributor", sc.base_dir);
            // spawn tasks that run in parallel
            let mut items = vec![];

            for (name, ac) in &sc.activities {
                items.push(activity::Activity {
                    name: name.clone(),
                    script: format!("{}/{}", scripts_dir, ac.script),
                    wants: ac.wants.clone(),
                    gpus: ac.gpus,
                    progress_regex: ac.progress_regex.clone(),
                    watch_dir: format!("{}/{}", sc.base_dir, name),
                    status_path: format!("{}/{}", status_dir, name),
                    queue_path: format!("{}/{}", queue_dir, name),
                    work_dir: work_dir.clone(),
                    distributor_dir: distributor_dir.clone()
                });
            }

            let tasks: Vec<_> = items
                .into_iter()
                .map(|item| {
                    tokio::spawn(async {
                        item.launch().await;
                        item
                    })
                })
                .collect();

            // await the tasks for resolve's to complete and give back our items
            let mut items = vec![];
            for task in tasks {
                items.push(task.await.unwrap());
            }
        }
        Action::Status => {
            print_for_users("/var/srvrs/status");
        }
        Action::Services => {
            unimplemented!();
            /*
            // Lol
            println!("Available Services:\nwhisper, an auto-captioning service for audio and video files");
            */
        }
        Action::Queue => {
            print_for_users("/var/srvrs/queue");
        }
    }
}

fn print_for_users(dir_path: &str) -> Result<(), Error> {
    let dir = fs::read_dir(dir_path)?;
    for file in dir {
        let file_handle = fs::File::open(file?.path());
        match file_handle {
            Ok(mut f) => {
                let mut contents = String::new();
                f.read_to_string(&mut contents)?;
                println!("{}", contents);
            },
            Err(e) => {
                error!("{}", e);
            },
        };
    }
    Ok(())
}
