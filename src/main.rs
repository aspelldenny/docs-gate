mod checks;
mod config;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

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
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let config = config::load_config(cli.config.as_deref());
    let results = checks::run_all_checks(&config);
    let output = output::format_results(&results, cli.verbose);
    println!("{output}");
    output::exit_code(&results)
}
