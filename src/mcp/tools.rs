use serde::Deserialize;

use crate::config::Config;

#[derive(Debug, Deserialize, schemars::JsonSchema, Default)]
pub struct DocsDirParam {
    /// Override docs directory path (uses config default if not provided)
    pub docs_dir: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FilePathParam {
    /// Path to the file to check (required)
    pub file_path: String,
}

pub fn resolve_config(base: &Config, docs_dir: Option<String>) -> Config {
    let mut config = Config {
        docs_dir: base.docs_dir.clone(),
        changelog: base.changelog.clone(),
        changelog_max_age_days: base.changelog_max_age_days,
        architecture: crate::config::ArchitectureConfig {
            enabled: base.architecture.enabled,
            file: base.architecture.file.clone(),
            required_sections: base.architecture.required_sections,
            required_non_empty: base.architecture.required_non_empty.clone(),
        },
        ticket: crate::config::TicketConfig {
            ticket_dir: base.ticket.ticket_dir.clone(),
            type_pattern: base.ticket.type_pattern.clone(),
            valid_types: base.ticket.valid_types.clone(),
            exclude_files: base.ticket.exclude_files.clone(),
        },
        changelog_staged: base.changelog_staged,
        rules: base.rules.clone(),
        staleness: base.staleness.clone(),
    };
    if let Some(dir) = docs_dir {
        config.docs_dir = dir.into();
    }
    config
}
