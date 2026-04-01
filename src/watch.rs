use std::process::ExitCode;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use tokio::sync::mpsc;

use crate::checks;
use crate::config::Config;
use crate::output;

const DEBOUNCE_MS: u64 = 500;

fn run_checks(config: &Config, extended: bool, verbose: bool) -> ExitCode {
    let results = if extended {
        checks::run_all_checks_extended(config)
    } else {
        checks::run_all_checks(config)
    };
    let out = output::format_results(&results, verbose);
    println!("{out}");
    output::exit_code(&results)
}

fn clear_and_timestamp() {
    let now = chrono::Local::now().format("%H:%M:%S");
    print!("\x1B[2J\x1B[H");
    println!("[{now}] Running checks...\n");
}

pub async fn run_watch(config: &Config, extended: bool) -> ExitCode {
    // Initial run
    clear_and_timestamp();
    let mut last_exit = run_checks(config, extended, true);

    // Setup file watcher
    let (tx, mut rx) = mpsc::channel(100);

    let mut watcher = match notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res
            && (event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove())
        {
            let _ = tx.blocking_send(());
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error: failed to start file watcher: {e}");
            return ExitCode::from(2);
        }
    };

    // Watch docs_dir
    if let Err(e) = watcher.watch(&config.docs_dir, RecursiveMode::Recursive) {
        eprintln!("Error: cannot watch {}: {e}", config.docs_dir.display());
        return ExitCode::from(2);
    }

    // Watch ticket_dir if extended
    if extended
        && let Err(e) = watcher.watch(&config.ticket.ticket_dir, RecursiveMode::Recursive)
    {
        eprintln!("Warning: cannot watch {}: {e}", config.ticket.ticket_dir.display());
        // Continue — ticket dir might not exist yet
    }

    // Event loop with debounce
    loop {
        tokio::select! {
            _ = rx.recv() => {
                // Debounce: drain any events that arrive within 500ms
                tokio::time::sleep(Duration::from_millis(DEBOUNCE_MS)).await;
                while rx.try_recv().is_ok() {}

                clear_and_timestamp();
                last_exit = run_checks(config, extended, true);
            }
            _ = tokio::signal::ctrl_c() => {
                return last_exit;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_debounce_constant() {
        assert_eq!(DEBOUNCE_MS, 500);
    }

    #[test]
    fn test_run_checks_nonexistent_dir_fails() {
        let mut config = Config::default();
        config.docs_dir = PathBuf::from("/tmp/docs-gate-nonexistent-dir-test");
        let exit = run_checks(&config, false, false);
        assert_eq!(exit, ExitCode::from(1));
    }

    #[test]
    fn test_run_checks_extended_nonexistent_dir_fails() {
        let mut config = Config::default();
        config.docs_dir = PathBuf::from("/tmp/docs-gate-nonexistent-dir-test");
        config.ticket.ticket_dir = PathBuf::from("/tmp/docs-gate-nonexistent-ticket-test");
        let exit = run_checks(&config, true, false);
        assert_eq!(exit, ExitCode::from(1));
    }

    #[test]
    fn test_clear_and_timestamp_no_panic() {
        // Just verify it doesn't panic
        clear_and_timestamp();
    }
}
