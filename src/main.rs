mod checks;
mod config;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// docs-gate: Check docs compliance before commit
#[derive(Parser, Debug)]
#[command(name = "docs-gate", version, about)]
struct Cli {
    /// Path to config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Show details for passing checks too
    #[arg(long)]
    verbose: bool,

    /// Run all checks including ticket type classification
    #[arg(long)]
    all: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check a file for Discovery Report format
    CheckDiscovery {
        /// Path to the file to check
        file: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let config = config::load_config(cli.config.as_deref());

    let results = match cli.command {
        Some(Commands::CheckDiscovery { ref file }) => checks::discovery::check_discovery(file),
        None if cli.all => checks::run_all_checks_extended(&config),
        None => checks::run_all_checks(&config),
    };

    let output = output::format_results(&results, cli.verbose);
    println!("{output}");
    output::exit_code(&results)
}
