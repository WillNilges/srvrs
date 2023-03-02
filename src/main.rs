// Example from https://github.com/notify-rs/notify/blob/0f5fcda7a0f02d19eb0660a7fe65303d74550cfc/examples/monitor_raw.rs

// To know what directories to watch
use clap::Parser;

pub mod srvrs;

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
    let service = srvrs::Srvrs { 
        primary_path: args.primary_path,
        work_path: args.work_path,
        command: args.command
    };
    service.launch();
}

