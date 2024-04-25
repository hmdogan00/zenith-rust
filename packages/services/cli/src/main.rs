use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE", default_value = "zenith.json")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, default_value = "0")]
    debug: Option<u8>,

    #[command(subcommand)]
    command: Commands,

    /// The path to the monorepo
    #[arg(short, long)]
    monorepo: String
}

#[derive(Subcommand)]
enum Commands {
    /// Lists affected projects and directories
    Affected(affected::AffectedArgs),
    /// Tries to retrieve the result of the command from cache, runs the command if not found and caches the result
    /// If the command fails, the cache is not updated
    Run(run::RunArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.config {
        Some(path) => println!("Using config file: {:?}", path),
        None => println!("Using default config file, zenith.json"),
    }

    let mut workspace = vec![];
    init::get_workspace(&cli.monorepo, &mut workspace).expect("Could not read workspace!");

    match cli.debug {
        Some(0) => {},
        _ => {
            println!("Debug mode is on");
            println!("Workspace: {:?}", workspace);
        },
    }

    match &cli.command {
        Commands::Affected(args ) => {
            affected::list_affected(args);
        }
        Commands::Run(args) => {
            run::run(&args, workspace);
        }
    }
}