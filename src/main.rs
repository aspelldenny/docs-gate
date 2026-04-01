mod checks;
mod config;
mod mcp;
mod output;
mod watch;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rmcp::ServiceExt;

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
    /// Start MCP server on stdio transport
    Serve,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // --watch + subcommands is not supported
    if cli.watch && cli.command.is_some() {
        eprintln!("Error: Watch mode not supported with subcommands");
        return ExitCode::from(2);
    }

    let config = config::load_config(cli.config.as_deref());

    match cli.command {
        Some(Commands::Serve) => {
            let server = mcp::server::DocsGateServer::new(config);
            let transport = rmcp::transport::stdio();
            match server.serve(transport).await {
                Ok(running) => {
                    let _ = running.waiting().await;
                    ExitCode::from(0)
                }
                Err(e) => {
                    eprintln!("Error: MCP server failed: {e}");
                    ExitCode::from(2)
                }
            }
        }
        _ if cli.watch => watch::run_watch(&config, cli.all).await,
        Some(Commands::CheckDiscovery { ref file }) => {
            let results = checks::discovery::check_discovery(file);
            let output = output::format_results(&results, cli.verbose);
            println!("{output}");
            output::exit_code(&results)
        }
        None if cli.all => {
            let results = checks::run_all_checks_extended(&config);
            let output = output::format_results(&results, cli.verbose);
            println!("{output}");
            output::exit_code(&results)
        }
        None => {
            let results = checks::run_all_checks(&config);
            let output = output::format_results(&results, cli.verbose);
            println!("{output}");
            output::exit_code(&results)
        }
    }
}
