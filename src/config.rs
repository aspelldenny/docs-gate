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
    /// Require CHANGELOG in every staged commit (not just "recent entry")
    #[serde(default = "default_true")]
    pub changelog_staged: bool,
    /// File-to-docs mapping rules: if watched file is staged, required doc must also be staged
    #[serde(default)]
    pub rules: Vec<RuleConfig>,
    /// Staleness checks: warn when a file hasn't been updated in N commits
    #[serde(default)]
    pub staleness: Vec<StalenessConfig>,
    /// Generic structural checks for additional doc files (beyond ARCHITECTURE.md).
    /// Each entry runs the same section-count + required-non-empty logic as
    /// `[architecture]` against the named file under `docs_dir`.
    #[serde(default, rename = "doc_structure")]
    pub doc_structure: Vec<DocStructureConfig>,
    /// Drift checks: extract a number from a doc, run a command, compare the
    /// command's output to the doc. Catches stale claims like "258/258 tests pass".
    #[serde(default, rename = "count_check")]
    pub count_check: Vec<CountCheckConfig>,
    /// Cross-doc consistency: target file must contain every value extracted from
    /// the source file. Catches drift between two docs that should agree.
    #[serde(default, rename = "cross_doc")]
    pub cross_doc: Vec<CrossDocConfig>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleConfig {
    /// Glob pattern for source files to watch (e.g. "src/lib/ai/prompts.ts", "src/app/api/**/*.ts")
    pub watch: String,
    /// Doc file that must be updated when watched files change (e.g. "docs/PROMPTS.md")
    pub requires: String,
    /// Optional human-readable message on failure
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StalenessConfig {
    /// File to monitor for staleness (e.g. "CLAUDE.md", "docs/PROJECT.md")
    pub file: String,
    /// Max commits since last update before triggering (default 20)
    #[serde(default = "default_max_commits")]
    pub max_commits: u32,
    /// "warn" (default) or "fail"
    #[serde(default = "default_warn")]
    pub level: String,
}

fn default_max_commits() -> u32 {
    20
}

fn default_warn() -> String {
    String::from("warn")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountCheckConfig {
    /// Doc file containing the hardcoded number. Resolved against `docs_dir` first,
    /// then against the project root.
    pub file: String,
    /// Regex with one capture group to extract the number from the doc
    pub doc_pattern: String,
    /// Shell command to run for the actual count
    pub command: String,
    /// Regex with one capture group to extract the number from command stdout
    pub command_pattern: String,
    /// Human-readable label shown in error messages (e.g. "test count")
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrossDocConfig {
    /// Source doc — declares the canonical set of values
    pub source: String,
    /// Regex with one capture group; all matches in source = canonical set
    pub source_pattern: String,
    /// Target doc — must contain every value from source
    pub target: String,
    /// Regex with one capture group; all matches in target = "what target has"
    pub target_pattern: String,
    /// Human-readable label shown in error messages
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocStructureConfig {
    /// Path to doc file relative to `docs_dir` (e.g. "TEST_CASES.md")
    pub file: String,
    /// Number of `## N.` headings required
    pub required_sections: usize,
    /// 1-indexed section numbers that must be non-empty
    #[serde(default)]
    pub required_non_empty: Vec<usize>,
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
            changelog_staged: true,
            rules: Vec::new(),
            staleness: Vec::new(),
            doc_structure: Vec::new(),
            count_check: Vec::new(),
            cross_doc: Vec::new(),
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
        assert!(config.changelog_staged);
        assert!(config.rules.is_empty());
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
    fn test_load_config_with_doc_structure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "[[doc_structure]]").unwrap();
        writeln!(f, "file = \"TEST_CASES.md\"").unwrap();
        writeln!(f, "required_sections = 3").unwrap();
        writeln!(f, "required_non_empty = [1, 2, 3]").unwrap();
        writeln!(f).unwrap();
        writeln!(f, "[[doc_structure]]").unwrap();
        writeln!(f, "file = \"AUDIT_PROTOCOL.md\"").unwrap();
        writeln!(f, "required_sections = 6").unwrap();

        let config = load_config(Some(&path));
        assert_eq!(config.doc_structure.len(), 2);
        assert_eq!(config.doc_structure[0].file, "TEST_CASES.md");
        assert_eq!(config.doc_structure[0].required_sections, 3);
        assert_eq!(config.doc_structure[0].required_non_empty, vec![1, 2, 3]);
        assert_eq!(config.doc_structure[1].file, "AUDIT_PROTOCOL.md");
        assert_eq!(config.doc_structure[1].required_sections, 6);
        assert!(config.doc_structure[1].required_non_empty.is_empty());
    }

    #[test]
    fn test_load_config_no_doc_structure_defaults_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".docs-gate.toml");
        std::fs::write(&path, "docs_dir = \"docs\"\n").unwrap();

        let config = load_config(Some(&path));
        assert!(config.doc_structure.is_empty());
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
