use std::collections::HashSet;
use std::process::Command;

use crate::checks::{CheckResult, CheckStatus};
use crate::config::Config;

/// Get list of staged files from git (git diff --cached --name-only).
/// Returns None if not in a git repo or git command fails.
fn get_staged_files() -> Option<HashSet<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: HashSet<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    Some(files)
}

/// Check if a glob pattern matches any of the staged files.
/// Supports simple patterns: "exact/path.ts", "dir/**/*.ts", "dir/*.ts"
fn glob_matches_any(pattern: &str, files: &HashSet<String>) -> bool {
    // Try exact match first (most common case)
    if files.contains(pattern) {
        return true;
    }

    // Build a glob matcher
    let matcher = match glob_to_regex(pattern) {
        Some(re) => re,
        None => return false,
    };

    files.iter().any(|f| matcher.is_match(f))
}

/// Convert a simple glob pattern to a regex.
/// Supports: * (single segment), ** (multi segment), ? (single char)
fn glob_to_regex(pattern: &str) -> Option<regex::Regex> {
    let mut re = String::from("^");
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next(); // consume second *
                    if chars.peek() == Some(&'/') {
                        chars.next(); // consume /
                        re.push_str("(.+/)?"); // match zero or more path segments
                    } else {
                        re.push_str(".*"); // ** at end = match everything
                    }
                } else {
                    re.push_str("[^/]*"); // * = match within single segment
                }
            }
            '?' => re.push_str("[^/]"),
            '.' => re.push_str(r"\."),
            '/' => re.push('/'),
            c => re.push(c),
        }
    }

    re.push('$');
    regex::Regex::new(&re).ok()
}

/// Check that CHANGELOG is in staged files (if changelog_staged is enabled).
pub fn check_changelog_staged(config: &Config) -> CheckResult {
    let name = String::from("changelog-staged");

    if !config.changelog_staged {
        return CheckResult {
            name,
            status: CheckStatus::Pass,
        };
    }

    let staged = match get_staged_files() {
        Some(files) => files,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Warn(String::from(
                    "Not a git repo or no staged files — skipping staged check",
                )),
            };
        }
    };

    if staged.is_empty() {
        // No staged files at all — not a commit context, skip
        return CheckResult {
            name,
            status: CheckStatus::Warn(String::from("No staged files — skipping")),
        };
    }

    // Check if any staged file is a code file (not just docs)
    let has_code_changes = staged.iter().any(|f| {
        !f.starts_with("docs/") && !f.starts_with(".") && f != "CLAUDE.md" && f != "README.md"
    });

    if !has_code_changes {
        // Only docs changes — no need to enforce changelog
        return CheckResult {
            name,
            status: CheckStatus::Pass,
        };
    }

    let changelog_path = format!("{}/{}", config.docs_dir.display(), config.changelog);

    if staged.contains(&changelog_path) {
        CheckResult {
            name,
            status: CheckStatus::Pass,
        }
    } else {
        CheckResult {
            name,
            status: CheckStatus::Fail(format!(
                "{} not in staged files — every code commit must update CHANGELOG",
                changelog_path,
            )),
        }
    }
}

/// Check file-to-docs mapping rules: if any watched file is staged,
/// the required doc must also be staged.
pub fn check_rules(config: &Config) -> Vec<CheckResult> {
    if config.rules.is_empty() {
        return Vec::new();
    }

    let staged = match get_staged_files() {
        Some(files) => files,
        None => {
            return vec![CheckResult {
                name: String::from("rules"),
                status: CheckStatus::Warn(String::from(
                    "Not a git repo or no staged files — skipping rules check",
                )),
            }];
        }
    };

    if staged.is_empty() {
        return Vec::new();
    }

    let mut results = Vec::new();

    for rule in &config.rules {
        // Check if any staged file matches the watch pattern
        if !glob_matches_any(&rule.watch, &staged) {
            continue; // This rule doesn't apply to current commit
        }

        // Watch pattern matched — check if required doc is also staged
        if staged.contains(&rule.requires) {
            results.push(CheckResult {
                name: format!("rule-{}", rule.requires),
                status: CheckStatus::Pass,
            });
        } else {
            let message = rule.message.clone().unwrap_or_else(|| {
                format!(
                    "'{}' changed but '{}' not updated",
                    rule.watch, rule.requires
                )
            });
            results.push(CheckResult {
                name: format!("rule-{}", rule.requires),
                status: CheckStatus::Fail(message),
            });
        }
    }

    results
}

/// Count commits since a file was last modified.
/// Uses: git log --oneline -- <file> | wc -l for total, then git log --oneline | wc -l for total commits.
/// Returns None if git fails or file was never committed.
fn commits_since_last_touch(file: &str) -> Option<u32> {
    // Get the hash of the last commit that touched this file
    let last_touch = Command::new("git")
        .args(["log", "-1", "--format=%H", "--", file])
        .output()
        .ok()?;

    if !last_touch.status.success() {
        return None;
    }

    let hash = String::from_utf8_lossy(&last_touch.stdout)
        .trim()
        .to_string();

    if hash.is_empty() {
        return None; // File was never committed
    }

    // Count commits since that hash (exclusive)
    let count = Command::new("git")
        .args(["rev-list", "--count", &format!("{hash}..HEAD")])
        .output()
        .ok()?;

    if !count.status.success() {
        return None;
    }

    String::from_utf8_lossy(&count.stdout)
        .trim()
        .parse::<u32>()
        .ok()
}

/// Check staleness of tracked files: warn/fail if file hasn't been updated in N commits.
pub fn check_staleness(config: &Config) -> Vec<CheckResult> {
    let mut results = Vec::new();

    for entry in &config.staleness {
        let name = format!("staleness-{}", entry.file);

        let commits = match commits_since_last_touch(&entry.file) {
            Some(c) => c,
            None => {
                results.push(CheckResult {
                    name,
                    status: CheckStatus::Warn(format!(
                        "{} — not tracked by git or never committed",
                        entry.file
                    )),
                });
                continue;
            }
        };

        if commits <= entry.max_commits {
            results.push(CheckResult {
                name,
                status: CheckStatus::Pass,
            });
        } else {
            let message = format!(
                "{} not updated in {} commits (threshold: {}) — review and update",
                entry.file, commits, entry.max_commits
            );
            let status = if entry.level == "fail" {
                CheckStatus::Fail(message)
            } else {
                CheckStatus::Warn(message)
            };
            results.push(CheckResult { name, status });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_exact_match() {
        let files: HashSet<String> = vec!["src/lib/ai/prompts.ts".to_string()]
            .into_iter()
            .collect();
        assert!(glob_matches_any("src/lib/ai/prompts.ts", &files));
        assert!(!glob_matches_any("src/lib/ai/other.ts", &files));
    }

    #[test]
    fn test_glob_star() {
        let files: HashSet<String> = vec![
            "src/app/api/reading/create/route.ts".to_string(),
            "src/app/api/match/list/route.ts".to_string(),
        ]
        .into_iter()
        .collect();
        assert!(!glob_matches_any("src/app/api/*.ts", &files)); // * doesn't cross /
    }

    #[test]
    fn test_glob_double_star() {
        let files: HashSet<String> = vec![
            "src/app/api/reading/create/route.ts".to_string(),
            "src/components/ui/Toast.tsx".to_string(),
        ]
        .into_iter()
        .collect();
        assert!(glob_matches_any("src/app/api/**/*.ts", &files));
        assert!(!glob_matches_any(
            "src/app/api/**/*.ts",
            &HashSet::from(["src/components/ui/Toast.tsx".to_string()])
        ));
    }

    #[test]
    fn test_glob_double_star_components() {
        let files: HashSet<String> =
            vec!["src/components/journey/MonthlyPortraitWidget.tsx".to_string()]
                .into_iter()
                .collect();
        assert!(glob_matches_any("src/components/**/*.tsx", &files));
    }

    #[test]
    fn test_glob_to_regex_basic() {
        let re = glob_to_regex("src/lib/ai/prompts.ts").unwrap();
        assert!(re.is_match("src/lib/ai/prompts.ts"));
        assert!(!re.is_match("src/lib/ai/other.ts"));
    }

    #[test]
    fn test_glob_to_regex_double_star_slash() {
        let re = glob_to_regex("src/app/api/**/*.ts").unwrap();
        assert!(re.is_match("src/app/api/reading/create/route.ts"));
        assert!(re.is_match("src/app/api/match/route.ts"));
        assert!(!re.is_match("src/lib/ai/prompts.ts"));
    }

    #[test]
    fn test_glob_to_regex_single_star() {
        let re = glob_to_regex("src/data/*.json").unwrap();
        assert!(re.is_match("src/data/spreads.json"));
        assert!(re.is_match("src/data/cards.json"));
        assert!(!re.is_match("src/data/nested/file.json"));
    }
}
