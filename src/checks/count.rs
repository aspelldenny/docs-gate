use std::path::{Path, PathBuf};
use std::process::Command;

use crate::checks::{CheckResult, CheckStatus};
use crate::config::{Config, CountCheckConfig};

/// Iterate `[[count_check]]` entries; for each, extract a number from the
/// referenced doc and compare it to a number parsed from a command's stdout.
pub fn check_counts(config: &Config) -> Vec<CheckResult> {
    config
        .count_check
        .iter()
        .map(|entry| run_one(config, entry))
        .collect()
}

fn run_one(config: &Config, entry: &CountCheckConfig) -> CheckResult {
    let label = if entry.description.is_empty() {
        entry.file.clone()
    } else {
        entry.description.clone()
    };
    let name = format!("count-{label}");

    let path = match resolve_path(&config.docs_dir, &entry.file) {
        Some(p) => p,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("File not found: {}", entry.file)),
            };
        }
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Cannot read {}: {e}", path.display())),
            };
        }
    };

    let doc_re = match regex::Regex::new(&entry.doc_pattern) {
        Ok(r) => r,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Invalid doc_pattern regex: {e}")),
            };
        }
    };

    let doc_value = match capture_first(&doc_re, &content) {
        Some(v) => v,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!(
                    "doc_pattern did not match in {}",
                    path.display()
                )),
            };
        }
    };

    let doc_num: u64 = match doc_value.parse() {
        Ok(n) => n,
        Err(_) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!(
                    "doc_pattern captured non-numeric value: {doc_value:?}"
                )),
            };
        }
    };

    let output = match run_command(&entry.command) {
        Ok(o) => o,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Failed to run command: {e}")),
            };
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let snippet: String = stderr.chars().take(200).collect();
        return CheckResult {
            name,
            status: CheckStatus::Fail(format!(
                "Command failed with exit {:?}: {}",
                output.status.code(),
                snippet.trim()
            )),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let cmd_re = match regex::Regex::new(&entry.command_pattern) {
        Ok(r) => r,
        Err(e) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Invalid command_pattern regex: {e}")),
            };
        }
    };

    let cmd_value = match capture_first(&cmd_re, &stdout) {
        Some(v) => v,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(String::from(
                    "command_pattern did not match in command output",
                )),
            };
        }
    };

    let cmd_num: u64 = match cmd_value.parse() {
        Ok(n) => n,
        Err(_) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!(
                    "command_pattern captured non-numeric value: {cmd_value:?}"
                )),
            };
        }
    };

    if doc_num == cmd_num {
        CheckResult {
            name,
            status: CheckStatus::Pass,
        }
    } else {
        CheckResult {
            name,
            status: CheckStatus::Fail(format!(
                "{label}: doc says {doc_num}, command says {cmd_num}"
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

fn capture_first(re: &regex::Regex, text: &str) -> Option<String> {
    re.captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(unix)]
fn run_command(command: &str) -> std::io::Result<std::process::Output> {
    Command::new("sh").arg("-c").arg(command).output()
}

#[cfg(windows)]
fn run_command(command: &str) -> std::io::Result<std::process::Output> {
    Command::new("cmd").arg("/C").arg(command).output()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{content}").unwrap();
        path
    }

    fn config_with(dir: &Path, entries: Vec<CountCheckConfig>) -> Config {
        Config {
            docs_dir: dir.to_path_buf(),
            count_check: entries,
            ..Config::default()
        }
    }

    #[test]
    fn test_empty_returns_no_results() {
        let config = Config::default();
        assert!(check_counts(&config).is_empty());
    }

    #[test]
    fn test_pass_when_numbers_match() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "DOC.md", "We have 42 items.\n");
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "DOC.md".into(),
                doc_pattern: r"(\d+) items".into(),
                command: "echo 'count: 42'".into(),
                command_pattern: r"count: (\d+)".into(),
                description: "items".into(),
            }],
        );
        let results = check_counts(&config);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass), "{results:?}");
        assert_eq!(results[0].name, "count-items");
    }

    #[test]
    fn test_fail_when_numbers_differ() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "DOC.md", "We have 42 items.\n");
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "DOC.md".into(),
                doc_pattern: r"(\d+) items".into(),
                command: "echo 'count: 7'".into(),
                command_pattern: r"count: (\d+)".into(),
                description: "items".into(),
            }],
        );
        let results = check_counts(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("doc says 42") && s.contains("command says 7"))
        );
    }

    #[test]
    fn test_fail_when_doc_pattern_no_match() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "DOC.md", "Nothing numeric here.\n");
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "DOC.md".into(),
                doc_pattern: r"(\d+) widgets".into(),
                command: "echo 5".into(),
                command_pattern: r"(\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_counts(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("doc_pattern did not match"))
        );
    }

    #[test]
    fn test_fail_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "NOT_THERE.md".into(),
                doc_pattern: r"(\d+)".into(),
                command: "echo 1".into(),
                command_pattern: r"(\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_counts(&config);
        assert!(matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("File not found")));
    }

    #[test]
    fn test_fail_when_command_exits_nonzero() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "DOC.md", "5 items\n");
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "DOC.md".into(),
                doc_pattern: r"(\d+) items".into(),
                command: "exit 3".into(),
                command_pattern: r"(\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_counts(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("Command failed"))
        );
    }

    #[test]
    fn test_fail_when_command_pattern_no_match() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "DOC.md", "5 items\n");
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "DOC.md".into(),
                doc_pattern: r"(\d+) items".into(),
                command: "echo 'no number here at all'".into(),
                command_pattern: r"count: (\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_counts(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("command_pattern did not match"))
        );
    }

    #[test]
    fn test_invalid_doc_regex_fails_fast() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "DOC.md", "anything\n");
        let config = config_with(
            dir.path(),
            vec![CountCheckConfig {
                file: "DOC.md".into(),
                doc_pattern: "(unclosed".into(),
                command: "echo 1".into(),
                command_pattern: r"(\d+)".into(),
                description: String::new(),
            }],
        );
        let results = check_counts(&config);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("Invalid doc_pattern"))
        );
    }
}
