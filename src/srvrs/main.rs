#![feature(path_file_prefix)]
use clap::{Args, Parser, Subcommand};
use std::{io::Read, fs::File};

pub mod srvrs;
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
    /// Path of dir to watch
    #[arg(short, long, required = true)]
    primary_path: String,

    /// Path where we do our work
    #[arg(short, long, required = true)]
    work_path: String,

    /// Command to run with path as argument
    #[arg(short, long, required = true)]
    command: String,

    /// Path to distributor directory
    #[arg(short, long, required = true)]
    distributor_path: String,
}

fn main() {
    let args = SubCommands::parse();
    match args.subcommand {
        Action::Watch(watch_args) => {
            let service = srvrs::Srvrs {
                primary_path: watch_args.primary_path,
                work_path: watch_args.work_path,
                command: watch_args.command,
                distributor_path: watch_args.distributor_path,
            };
            service.launch();
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
