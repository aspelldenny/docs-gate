pub mod architecture;
pub mod changelog;
pub mod count;
pub mod cross_doc;
pub mod discovery;
pub mod staged;
pub mod ticket;

use schemars::JsonSchema;
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Serialize, JsonSchema)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    Warn(String),
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
}

pub fn run_all_checks(config: &Config) -> Vec<CheckResult> {
    let mut results = Vec::new();
    results.push(changelog::check_changelog(config));
    results.extend(architecture::check_architecture(config));
    // Generic structural checks for additional doc files (no-op when [[doc_structure]] is empty)
    results.extend(architecture::check_doc_structure(config));
    // Git-aware checks: staged changelog + file-to-docs rules + staleness
    results.push(staged::check_changelog_staged(config));
    results.extend(staged::check_rules(config));
    results.extend(staged::check_staleness(config));
    // Drift checks: doc claims vs command output / cross-doc consistency
    // (both no-ops when their config arrays are empty)
    results.extend(count::check_counts(config));
    results.extend(cross_doc::check_cross_doc(config));
    results
}

pub fn run_all_checks_extended(config: &Config) -> Vec<CheckResult> {
    let mut results = run_all_checks(config);
    results.extend(ticket::check_tickets(config));
    results
}
