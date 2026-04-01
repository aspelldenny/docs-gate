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
        architecture: base.architecture.clone(),
        required_sections: base.required_sections,
        required_non_empty: base.required_non_empty.clone(),
        changelog_max_age_days: base.changelog_max_age_days,
        ticket: crate::config::TicketConfig {
            ticket_dir: base.ticket.ticket_dir.clone(),
            valid_types: base.ticket.valid_types.clone(),
            exclude_files: base.ticket.exclude_files.clone(),
        },
    };
    if let Some(dir) = docs_dir {
        config.docs_dir = dir.into();
    }
    config
}
