// Example from https://github.com/notify-rs/notify/blob/0f5fcda7a0f02d19eb0660a7fe65303d74550cfc/examples/monitor_raw.rs

// To watch directories
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::{path::{Path, PathBuf}, fs::{self, copy}};
use file_owner::PathExt;

// To know what directories to watch
use clap::Parser;

// To run commands based on said directories
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of dir to watch
    #[arg(short, long)]
    primary_path: String,

    /// Path where we do our work
    #[arg(short, long)]
    work_path: String,

    /// Command to run with path as argument 
    #[arg(short, long)]
    command: String,
}

fn main() {
    let args = Args::parse();
    /* FIXME: This could be really dangerous.
    println!("Clearing {}", &args.primary_path);
    fs::remove_dir_all(&args.primary_path);
    fs::create_dir(&args.primary_path);
    */
    println!("watching {}", args.primary_path);
    if let Err(e) = watch(args.primary_path, args.work_path, args.command) {
        println!("error: {:?}", e)
    }
}

fn watch<P: AsRef<Path>>(path: P, work_path: String, command: String) -> notify::Result<()> {
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
                        println!("changed: {:?}", event);
                        // TODO: Make this app work with multiple paths at once
                        //println!("{:?}", event.paths); // Debug for seeing event info
                        respond(&command, &work_path, event.paths);
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

fn respond(command: &String, work_path: &String, files: Vec<PathBuf>) {

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
    let new_user_work_dir = format!("{}/{}_{}", work_path, owner, first_file_name_prefix);
    println!("Creating {} for new user work.", new_user_work_dir);
    fs::create_dir(&new_user_work_dir)
        .unwrap_or_else(|e| panic!("Error creating dir: {}", e));

    // Move file into temp work directory
    let new_user_file_path = format!("{}/{}", new_user_work_dir, first_file_name);
    fs::rename(first_file, &new_user_file_path)
        .unwrap_or_else(|e| panic!("Error copying file: {}", e));

    println!("Running command!");

    let built_command = command.to_owned() + &new_user_file_path;
    let output = Command::new("sh")
                .arg("-c")
                .arg(built_command)
                .output()
                .expect("failed to execute process");

    let hello = output.stdout;
    println!("{}", String::from_utf8_lossy(&hello));
}
