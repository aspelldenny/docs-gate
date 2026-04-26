use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::checks::{CheckResult, CheckStatus};
use crate::config::{Config, CrossDocConfig};

/// Iterate `[[cross_doc]]` entries; each one verifies that every value extracted
/// from `source` (via `source_pattern`) also appears in `target` (via
/// `target_pattern`). Subset relationship — target may contain extra values.
pub fn check_cross_doc(config: &Config) -> Vec<CheckResult> {
    config
        .cross_doc
        .iter()
        .map(|entry| run_one(config, entry))
        .collect()
}

fn run_one(config: &Config, entry: &CrossDocConfig) -> CheckResult {
    let label = if entry.description.is_empty() {
        format!("{}-{}", entry.source, entry.target)
    } else {
        entry.description.clone()
    };
    let name = format!("cross-{label}");

    let source_path = match resolve_path(&config.docs_dir, &entry.source) {
        Some(p) => p,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Source file not found: {}", entry.source)),
            };
        }
    };
    let target_path = match resolve_path(&config.docs_dir, &entry.target) {
        Some(p) => p,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Target file not found: {}", entry.target)),
            };
        }
    };

    let source_content = match std::fs::read_to_string(&source_path) {
        Ok(c) => c,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Cannot read {}: {e}", source_path.display())),
            };
        }
    };
    let target_content = match std::fs::read_to_string(&target_path) {
        Ok(c) => c,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Cannot read {}: {e}", target_path.display())),
            };
        }
    };

    let source_re = match regex::Regex::new(&entry.source_pattern) {
        Ok(r) => r,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Invalid source_pattern regex: {e}")),
            };
        }
    };
    let target_re = match regex::Regex::new(&entry.target_pattern) {
        Ok(r) => r,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Invalid target_pattern regex: {e}")),
            };
        }
    };

    let source_set = collect_captures(&source_re, &source_content);
    let target_set = collect_captures(&target_re, &target_content);

    if source_set.is_empty() {
        return CheckResult {
            name,
            status: CheckStatus::Warn(format!(
                "source_pattern matched nothing in {}",
                source_path.display()
            )),
        };
    }

    let missing: Vec<String> = source_set
        .difference(&target_set)
        .cloned()
        .collect::<Vec<_>>();

    if missing.is_empty() {
        CheckResult {
            name,
            status: CheckStatus::Pass,
        }
    } else {
        let mut sorted = missing;
        sorted.sort();
        let preview: Vec<String> = sorted.iter().take(5).cloned().collect();
        let suffix = if sorted.len() > 5 {
            format!(" and {} more", sorted.len() - 5)
        } else {
            String::new()
        };
        CheckResult {
            name,
            status: CheckStatus::Fail(format!(
                "{label}: target missing values from source: {}{suffix}",
                preview.join(", ")
            )),
        }
    }
}

fn resolve_path(docs_dir: &Path, file: &str) -> Option<PathBuf> {
    let path = Path::new(file);
    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }
    let in_docs = docs_dir.join(file);
    if in_docs.exists() {
        return Some(in_docs);
    }
    let in_cwd = PathBuf::from(file);
    if in_cwd.exists() {
        return Some(in_cwd);
    }
    None
}

fn collect_captures(re: &regex::Regex, text: &str) -> HashSet<String> {
    re.captures_iter(text)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    fn config_with(dir: &Path, entries: Vec<CrossDocConfig>) -> Config {
        Config {
            docs_dir: dir.to_path_buf(),
            cross_doc: entries,
            ..Config::default()
        }
    }

    #[test]
    fn test_empty_returns_no_results() {
        let config = Config::default();
        assert!(check_cross_doc(&config).is_empty());
    }

    #[test]
    fn test_pass_when_target_contains_all_source_values() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "SRC.md", "Phase 1\nPhase 2\nPhase 3\n");
        write_file(dir.path(), "TGT.md", "Phase 1 done\nPhase 2 done\nPhase 3 done\nPhase 4 future\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "SRC.md".into(),
                source_pattern: r"Phase (\d+)".into(),
                target: "TGT.md".into(),
                target_pattern: r"Phase (\d+)".into(),
                description: "phases".into(),
            }],
        );
        let results = check_cross_doc(&config);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass), "{results:?}");
        assert_eq!(results[0].name, "cross-phases");
    }

    #[test]
    fn test_fail_when_target_missing_values() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "SRC.md", "Phase 1\nPhase 2\nPhase 3\n");
        write_file(dir.path(), "TGT.md", "Phase 1\nPhase 2\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "SRC.md".into(),
                source_pattern: r"Phase (\d+)".into(),
                target: "TGT.md".into(),
                target_pattern: r"Phase (\d+)".into(),
                description: "phases".into(),
            }],
        );
        let results = check_cross_doc(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("3") && s.contains("missing"))
        );
    }

    #[test]
    fn test_warn_when_source_pattern_matches_nothing() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "SRC.md", "Nothing matches.\n");
        write_file(dir.path(), "TGT.md", "Phase 1\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "SRC.md".into(),
                source_pattern: r"Phase (\d+)".into(),
                target: "TGT.md".into(),
                target_pattern: r"Phase (\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_cross_doc(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Warn(ref s) if s.contains("source_pattern matched nothing"))
        );
    }

    #[test]
    fn test_fail_when_source_missing() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "TGT.md", "anything\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "MISSING.md".into(),
                source_pattern: r"(\d+)".into(),
                target: "TGT.md".into(),
                target_pattern: r"(\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_cross_doc(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("Source file not found"))
        );
    }

    #[test]
    fn test_fail_when_target_missing() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "SRC.md", "Phase 1\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "SRC.md".into(),
                source_pattern: r"Phase (\d+)".into(),
                target: "MISSING.md".into(),
                target_pattern: r"Phase (\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_cross_doc(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("Target file not found"))
        );
    }

    #[test]
    fn test_invalid_source_regex_fails_fast() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "SRC.md", "x\n");
        write_file(dir.path(), "TGT.md", "x\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "SRC.md".into(),
                source_pattern: "(unclosed".into(),
                target: "TGT.md".into(),
                target_pattern: r"(\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_cross_doc(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("Invalid source_pattern"))
        );
    }

    #[test]
    fn test_missing_list_truncated_at_five() {
        let dir = tempfile::tempdir().unwrap();
        write_file(
            dir.path(),
            "SRC.md",
            "x:1\nx:2\nx:3\nx:4\nx:5\nx:6\nx:7\n",
        );
        write_file(dir.path(), "TGT.md", "nothing\n");
        let config = config_with(
            dir.path(),
            vec![CrossDocConfig {
                source: "SRC.md".into(),
                source_pattern: r"x:(\d+)".into(),
                target: "TGT.md".into(),
                target_pattern: r"x:(\d+)".into(),
                description: "items".into(),
            }],
        );
        let results = check_cross_doc(&config);
        let msg = match &results[0].status {
            CheckStatus::Fail(s) => s.clone(),
            other => panic!("expected Fail, got {other:?}"),
        };
        assert!(msg.contains("and 2 more"), "msg was: {msg}");
    }
}
