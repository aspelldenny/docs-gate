use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub docs_dir: PathBuf,
    pub changelog: String,
    pub changelog_max_age_days: u32,
    pub architecture: ArchitectureConfig,
    pub ticket: TicketConfig,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ArchitectureConfig {
    pub enabled: bool,
    pub file: String,
    pub required_sections: usize,
    pub required_non_empty: Vec<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TicketConfig {
    pub ticket_dir: PathBuf,
    pub type_pattern: String,
    pub valid_types: Vec<String>,
    pub exclude_files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            docs_dir: PathBuf::from("docs"),
            changelog: String::from("CHANGELOG.md"),
            changelog_max_age_days: 1,
            architecture: ArchitectureConfig::default(),
            ticket: TicketConfig::default(),
        }
    }
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            file: String::from("ARCHITECTURE.md"),
            required_sections: 9,
            required_non_empty: vec![7, 8, 9],
        }
    }
}

impl Default for TicketConfig {
    fn default() -> Self {
        Self {
            ticket_dir: PathBuf::from("docs/ticket"),
            type_pattern: String::from(r"\*\*Type:\*\*\s*`([^`]+)`"),
            valid_types: vec![
                String::from("read-only"),
                String::from("mutating"),
                String::from("destructive"),
            ],
            exclude_files: vec![String::from("TEMPLATE.md")],
        }
    }
}

pub fn load_config(path: Option<&Path>) -> Config {
    let config_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".docs-gate.toml"));

    if !config_path.exists() {
        return Config::default();
    }

    match std::fs::read_to_string(&config_path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Warning: failed to parse {}: {}", config_path.display(), e);
                Config::default()
            }
        },
        Err(e) => {
            eprintln!("Warning: cannot read {}: {}", config_path.display(), e);
            Config::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.docs_dir, PathBuf::from("docs"));
        assert_eq!(config.changelog, "CHANGELOG.md");
        assert!(config.architecture.enabled);
        assert_eq!(config.architecture.file, "ARCHITECTURE.md");
        assert_eq!(config.architecture.required_sections, 9);
        assert_eq!(config.architecture.required_non_empty, vec![7, 8, 9]);
        assert_eq!(config.changelog_max_age_days, 1);
        assert_eq!(config.ticket.ticket_dir, PathBuf::from("docs/ticket"));
        assert!(!config.ticket.type_pattern.is_empty());
        assert_eq!(
            config.ticket.valid_types,
            vec!["read-only", "mutating", "destructive"]
        );
        assert_eq!(config.ticket.exclude_files, vec!["TEMPLATE.md"]);
    }

    #[test]
    fn test_load_config_no_file() {
        let config = load_config(Some(Path::new("/tmp/nonexistent-docs-gate.toml")));
        assert_eq!(config.architecture.required_sections, 9);
    }

    #[test]
    fn test_load_config_valid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "docs_dir = \"documentation\"").unwrap();
        writeln!(f, "[architecture]").unwrap();
        writeln!(f, "required_sections = 5").unwrap();

        let config = load_config(Some(&path));
        assert_eq!(config.docs_dir, PathBuf::from("documentation"));
        assert_eq!(config.architecture.required_sections, 5);
        assert_eq!(config.changelog, "CHANGELOG.md");
    }

    #[test]
    fn test_load_config_architecture_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[architecture]").unwrap();
        writeln!(f, "enabled = false").unwrap();

        let config = load_config(Some(&path));
        assert!(!config.architecture.enabled);
    }

    #[test]
    fn test_load_config_with_ticket_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "docs_dir = \"docs\"").unwrap();
        writeln!(f, "[ticket]").unwrap();
        writeln!(f, "ticket_dir = \"custom/tickets\"").unwrap();
        writeln!(f, "exclude_files = [\"TEMPLATE.md\", \"DRAFT.md\"]").unwrap();
        writeln!(f, r#"type_pattern = '\*\*Loại:\*\*\s*`([^`]+)`'"#).unwrap();

        let config = load_config(Some(&path));
        assert_eq!(config.ticket.ticket_dir, PathBuf::from("custom/tickets"));
        assert_eq!(config.ticket.exclude_files, vec!["TEMPLATE.md", "DRAFT.md"]);
        assert_eq!(config.ticket.type_pattern, r"\*\*Loại:\*\*\s*`([^`]+)`");
        assert_eq!(config.docs_dir, PathBuf::from("docs"));
    }

    #[test]
    fn test_load_config_ticket_no_type_pattern() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[ticket]").unwrap();
        writeln!(f, "ticket_dir = \"docs/ticket\"").unwrap();

        let config = load_config(Some(&path));
        // When ticket section exists but type_pattern not set, uses default
        assert!(!config.ticket.type_pattern.is_empty());
    }

    #[test]
    fn test_load_config_no_ticket_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "docs_dir = \"docs\"").unwrap();

        let config = load_config(Some(&path));
        assert_eq!(config.ticket.ticket_dir, PathBuf::from("docs/ticket"));
        assert_eq!(
            config.ticket.valid_types,
            vec!["read-only", "mutating", "destructive"]
        );
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        std::fs::write(&path, "invalid = [[[").unwrap();

        let config = load_config(Some(&path));
        assert_eq!(config.architecture.required_sections, 9);
    }
}
