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

/// Take ownership of the freshly-loaded config and apply the optional per-call
/// `docs_dir` override. We take by value because each MCP tool call now loads its
/// own `Config` from disk (see `DocsGateServer::load_fresh_config`), so cloning
/// here would just be wasted work.
pub fn resolve_config(mut config: Config, docs_dir: Option<String>) -> Config {
    if let Some(dir) = docs_dir {
        config.docs_dir = dir.into();
    }
    config
}
