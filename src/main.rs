mod checks;
mod config;
mod output;
mod watch;

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

    /// Watch mode: re-run checks on file changes
    #[arg(long)]
    watch: bool,

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // --watch + check-discovery is not supported
    if cli.watch && cli.command.is_some() {
        eprintln!("Error: Watch mode not supported with check-discovery subcommand");
        return ExitCode::from(2);
    }

    let config = config::load_config(cli.config.as_deref());

    if cli.watch {
        return watch::run_watch(&config, cli.all).await;
    }

    let results = match cli.command {
        Some(Commands::CheckDiscovery { ref file }) => checks::discovery::check_discovery(file),
        None if cli.all => checks::run_all_checks_extended(&config),
        None => checks::run_all_checks(&config),
    };

    let output = output::format_results(&results, cli.verbose);
    println!("{output}");
    output::exit_code(&results)
}
