// To know what directories to watch
use clap::Parser;

pub mod distributor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path where we do our work
    #[arg(short, long, required=true)]
    work_path: String,

    /// Path where we put the finished product 
    #[arg(short, long, required=true)]
    destination_base_path: String,
}

fn main() {
    let args = Args::parse();
    let service = distributor::Distributor { 
        work_path: args.work_path,
        destination_base_path: args.destination_base_path
    };
    service.launch();
}
