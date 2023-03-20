#![feature(path_file_prefix)]
#![feature(unix_chown)]
use clap::{Args, Parser, Subcommand};
use std::{io::Read, fs, os::unix::fs::chown};
use serde_yaml;
use std::os::unix::fs::PermissionsExt;
use tokio;
use log::{error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;
use users::get_group_by_name;

pub mod activity;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct SubCommands {
    #[command(subcommand)]
    subcommand: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
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

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let args = SubCommands::parse();
    match args.subcommand {
        Action::Watch(watch_args) => {
            SimpleLogger::new().init().unwrap();
            log::set_max_level(LevelFilter::Info);

            let config = fs::read_to_string(watch_args.config_file).unwrap();
            let sc: activity::SrvrsConfig = serde_yaml::from_str(&config).unwrap();

            // All the required directories
            //let watch_dir = format!("{}/watch", sc.base_dir);
            let scripts_dir = format!("{}/scripts", sc.base_dir);
            let status_dir = format!("{}/status", sc.base_dir);
            let queue_dir = format!("{}/queue", sc.base_dir);
            let work_dir = format!("{}/work", sc.base_dir);
            let distributor_dir = format!("{}/distributor", sc.base_dir);

            for dir in vec![&scripts_dir, &work_dir, &distributor_dir] {
                info!("Creating directory: {}", &dir);
                fs::create_dir_all(&dir).unwrap();
                fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
            }

            for dir in vec![&status_dir, &queue_dir] {
                info!("Creating directory: {}", &dir);
                fs::create_dir_all(&dir).unwrap();
                fs::set_permissions(&dir, fs::Permissions::from_mode(0o740)).unwrap();

                let members_gid: u32 = match get_group_by_name("member") {
                        Some(group) => group.gid(),
                        _ => panic!("Group not found >:("),
                    };
                chown(dir, None, Some(members_gid)).unwrap();
            }

            // spawn tasks that run in parallel
            let mut items = vec![];

            for (name, ac) in &sc.activities {
                items.push(activity::Activity {
                    name: name.clone(),
                    script: format!("{}/{}", scripts_dir, name),
                    wants: ac.wants.clone(),
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
                .map(|mut item| {
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
            let file = fs::File::open("/var/srvrs/status");
            match file {
                Ok(mut f) => {
                    let mut contents = String::new();
                    f.read_to_string(&mut contents).unwrap();
                    println!("{}", contents);
                },
                Err(e) => {
                    println!("No Status: {}", e);
                }
            }
        }
        Action::Services => {
            // Lol
            println!("Available Services:\nwhisper, an auto-captioning service for audio and video files");
        }
        Action::Queue => {
            let file = fs::File::open("/var/srvrs/queue");
            match file {
                Ok(mut f) => {
                    let mut contents = String::new();
                    f.read_to_string(&mut contents).unwrap();
                    println!("{}", contents);
                },
                Err(e) => {
                    println!("No Queue: {}", e);
                }
            }
        }
    }
}
