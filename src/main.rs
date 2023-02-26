// Example from https://github.com/notify-rs/notify/blob/0f5fcda7a0f02d19eb0660a7fe65303d74550cfc/examples/monitor_raw.rs

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::path::Path;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Path of dir to watch
   #[arg(short, long)]
   path: String,

   /// Command to run with path as argument 
   #[arg(short, long)]
   command: String,
}

fn main() {
    let args = Args::parse();
    println!("watching {}", args.path);
    if let Err(e) = watch(args.path) {
        println!("error: {:?}", e)
    }
}

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => println!("changed: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}
