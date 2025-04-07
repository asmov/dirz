use std::path::{PathBuf};
use clap::{Parser, Subcommand};
use dirz_lib::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists all records in a directory
    List {
        #[clap(value_parser)]
        path: String
    },
    /// Displays the name of the current working cabinet root
    Which {},
    /// Displays the name of the current working cabinet
    Where {}
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { path } => cmd_list_directory(&PathBuf::from(path)),
        Commands::Which {} => cmd_cabinet_root_name(),
        Commands::Where {} => cmd_cabinet_name()
    }
}
