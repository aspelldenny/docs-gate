pub mod architecture;
pub mod changelog;
pub mod discovery;
pub mod ticket;

use crate::config::Config;

#[derive(Debug)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Warn(String),
}

#[derive(Debug)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
}

pub fn run_all_checks(config: &Config) -> Vec<CheckResult> {
    let mut results = Vec::new();
    results.push(changelog::check_changelog(config));
    results.extend(architecture::check_architecture(config));
    results
}

pub fn run_all_checks_extended(config: &Config) -> Vec<CheckResult> {
    let mut results = run_all_checks(config);
    results.extend(ticket::check_tickets(config));
    results
}
