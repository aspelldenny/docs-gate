pub mod architecture;
pub mod changelog;

use crate::config::Config;

#[derive(Debug)]
#[allow(dead_code)]
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
