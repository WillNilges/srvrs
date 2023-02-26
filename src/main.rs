// Example from https://github.com/notify-rs/notify/blob/0f5fcda7a0f02d19eb0660a7fe65303d74550cfc/examples/monitor_raw.rs

// To watch directories
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::{path::{Path, PathBuf}, fs};

// To know what directories to watch
use clap::Parser;

// To run commands based on said directories
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Path of dir to watch
   #[arg(short, long)]
   home_path: String,

   /// Command to run with path as argument 
   #[arg(short, long)]
   command: String,
}

fn main() {
    let args = Args::parse();
    /* FIXME: This could be really dangerous.
    println!("Clearing {}", &args.home_path);
    fs::remove_dir_all(&args.home_path);
    fs::create_dir(&args.home_path);
    */
    println!("watching {}", args.home_path);
    if let Err(e) = watch(args.home_path, args.command) {
        println!("error: {:?}", e)
    }
}

fn watch<P: AsRef<Path>>(path: P, command: String) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                //println!("changed: {:?}", event);
                match event.kind {
                    notify::EventKind::Create(notify::event::CreateKind::File) => {
                        // TODO: Make this app work with multiple paths at once
                        println!("{:?}", event.paths);
                        respond(&command, event.paths);
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

fn respond(command: &String, files: Vec<PathBuf>) {
    //fs::create_dir(home_path + "work");
    let built_command = command.to_owned() + &files[0].display().to_string();
    let output = Command::new("sh")
                .arg("-c")
                .arg(built_command)
                .output()
                .expect("failed to execute process");

    let hello = output.stdout;
    println!("{}", String::from_utf8_lossy(&hello));
}
