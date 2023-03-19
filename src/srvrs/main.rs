#![feature(path_file_prefix)]
use clap::{Args, Parser, Subcommand};
use std::{io::Read, fs::File};

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
    //#[arg(short, long, required = true)]
    //config_file: String,

    /// Path where we do our work
    #[arg(short, long, required = true)]
    work_path: String,

    /// Path to distributor directory
    #[arg(short, long, required = true)]
    distributor_path: String,
}

fn main() {
    let args = SubCommands::parse();
    match args.subcommand {
        Action::Watch(watch_args) => {
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

            //service.launch();
        }
        Action::Status => {
            let file = File::open("/var/srvrs/status");
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
            let file = File::open("/var/srvrs/queue");
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
