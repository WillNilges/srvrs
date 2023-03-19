#![feature(path_file_prefix)]
use clap::{Args, Parser, Subcommand};
use std::{io::Read, fs};
use serde_yaml;

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

fn main() {
    let args = SubCommands::parse();
    match args.subcommand {
        Action::Watch(watch_args) => {
            let config = fs::read_to_string(watch_args.config_file).unwrap();
            let sc: activity::SrvrsConfig = serde_yaml::from_str(&config).unwrap();

            // All the required directories
            let watch_dir = format!("{}/watch", sc.base_dir);
            let scripts_dir = format!("{}/scripts", sc.base_dir);
            let status_dir = format!("{}/status", sc.base_dir);
            let queue_dir = format!("{}/queue", sc.base_dir);
            let work_dir = format!("{}/work", sc.base_dir);
            let distributor_dir = format!("{}/distributor", sc.base_dir);

            /*
            let service = activity::Activity {
                name: "whisper".to_string(),
                script: "/var/srvrs/scripts/whsiper.sh".to_string(), 
                wants: vec![infer::MatcherType::Audio, infer::MatcherType::Video],
                progress_regex: r"([0-9][0-9]:[0-9][0-9].[0-9][0-9][0-9])( -->)".to_string(),
                watch_dir: "/var/srvrs/watch/whisper".to_string(),
                status_path: "/var/srvrs/status/whisper".to_string(),
                queue_path: "/var/srvrs/queue/whisper".to_string(),
                work_dir: "/var/srvrs/work".to_string(),
                distributor_dir: "/var/srvrs/distributor/".to_string(),
            };
            */
            for (name, ac) in &sc.activities {
                let activity = activity::Activity {
                    name: name.clone(),
                    script: format!("{}/{}", scripts_dir, name),
                    wants: ac.wants.clone(),
                    progress_regex: ac.progress_regex.clone(),
                    watch_dir: format!("{}/{}", watch_dir, name),
                    status_path: format!("{}/{}", status_dir, name),
                    queue_path: format!("{}/{}", queue_dir, name),
                    work_dir: work_dir.clone(),
                    distributor_dir: distributor_dir.clone()
                };
                activity.launch();
                println!("Activity Launched!");
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
