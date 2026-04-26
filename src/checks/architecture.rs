use std::path::Path;

use crate::checks::{CheckResult, CheckStatus};
use crate::config::Config;

/// Generic structural check: read a markdown file, count `## N.` headings, and
/// verify the requested sections are present and non-empty. Used by both the
/// `[architecture]` check and the `[[doc_structure]]` array.
pub fn check_doc_file(
    path: &Path,
    required_sections: usize,
    required_non_empty: &[usize],
    name_prefix: &str,
) -> Vec<CheckResult> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return vec![CheckResult {
                name: String::from(name_prefix),
                status: CheckStatus::Fail(format!("File not found: {}", path.display())),
            }];
        }
    };

    let re_section = regex::Regex::new(r"^## (\d+)\.").unwrap();
    let lines: Vec<&str> = content.lines().collect();

    let mut sections: Vec<(usize, usize)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = re_section.captures(line)
            && let Ok(num) = caps[1].parse::<usize>()
        {
            sections.push((i, num));
        }
    }

    let mut results = Vec::new();

    let unique_sections: std::collections::HashSet<usize> =
        sections.iter().map(|(_, n)| *n).collect();
    let found = unique_sections.len();

    if found < required_sections {
        results.push(CheckResult {
            name: format!("{name_prefix}-section-count"),
            status: CheckStatus::Fail(format!("Found {found}/{required_sections} sections")),
        });
    } else {
        results.push(CheckResult {
            name: format!("{name_prefix}-section-count"),
            status: CheckStatus::Pass,
        });
    }

    for &section_num in required_non_empty {
        let section_pos = sections.iter().find(|(_, n)| *n == section_num);

        match section_pos {
            None => {
                results.push(CheckResult {
                    name: format!("{name_prefix}-section-{section_num}"),
                    status: CheckStatus::Fail(format!("Section {section_num} missing")),
                });
            }
            Some(&(line_idx, _)) => {
                let next_heading = sections
                    .iter()
                    .find(|(i, _)| *i > line_idx)
                    .map(|(i, _)| *i)
                    .unwrap_or(lines.len());

                let has_content = lines[line_idx + 1..next_heading].iter().any(|l| {
                    let trimmed = l.trim();
                    !trimmed.is_empty()
                        && !trimmed.starts_with("<!--")
                        && !trimmed.starts_with("---")
                });

                if has_content {
                    results.push(CheckResult {
                        name: format!("{name_prefix}-section-{section_num}"),
                        status: CheckStatus::Pass,
                    });
                } else {
                    results.push(CheckResult {
                        name: format!("{name_prefix}-section-{section_num}"),
                        status: CheckStatus::Fail(format!("Section {section_num} empty")),
                    });
                }
            }
        }
    }

    results
}

pub fn check_architecture(config: &Config) -> Vec<CheckResult> {
    let name_prefix = "architecture";

    if !config.architecture.enabled {
        return vec![CheckResult {
            name: String::from(name_prefix),
            status: CheckStatus::Pass,
        }];
    }

    let path = config.docs_dir.join(&config.architecture.file);
    check_doc_file(
        &path,
        config.architecture.required_sections,
        &config.architecture.required_non_empty,
        name_prefix,
    )
}

/// Run `check_doc_file` for every `[[doc_structure]]` entry in config. Returns
/// an empty vec when no entries are configured (default).
pub fn check_doc_structure(config: &Config) -> Vec<CheckResult> {
    let mut results = Vec::new();
    for entry in &config.doc_structure {
        let path = config.docs_dir.join(&entry.file);
        let prefix = format!("doc-{}", entry.file);
        results.extend(check_doc_file(
            &path,
            entry.required_sections,
            &entry.required_non_empty,
            &prefix,
        ));
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn config_with_dir(dir: &std::path::Path) -> Config {
        Config {
            docs_dir: dir.to_path_buf(),
            architecture: crate::config::ArchitectureConfig::default(),
            ..Config::default()
        }
    }

    fn write_arch(dir: &std::path::Path, content: &str) {
        let path = dir.join("ARCHITECTURE.md");
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    fn full_9_sections() -> String {
        let mut s = String::from("# ARCHITECTURE\n\n");
        for i in 1..=9 {
            s.push_str(&format!(
                "## {i}. Section {i}\n\nContent for section {i}.\n\n"
            ));
        }
        s
    }

    #[test]
    fn test_pass_full_9_sections() {
        let dir = tempfile::tempdir().unwrap();
        write_arch(dir.path(), &full_9_sections());
        let results = check_architecture(&config_with_dir(dir.path()));
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, CheckStatus::Pass))
        );
    }

    #[test]
    fn test_fail_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let results = check_architecture(&config_with_dir(dir.path()));
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.starts_with("File not found"))
        );
    }

    #[test]
    fn test_fail_missing_sections() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::from("# ARCH\n\n");
        for i in 1..=5 {
            content.push_str(&format!("## {i}. Section\n\nContent.\n\n"));
        }
        write_arch(dir.path(), &content);
        let results = check_architecture(&config_with_dir(dir.path()));
        let count_result = results
            .iter()
            .find(|r| r.name == "architecture-section-count")
            .unwrap();
        assert!(
            matches!(count_result.status, CheckStatus::Fail(ref s) if s == "Found 5/9 sections")
        );
    }

    #[test]
    fn test_fail_section_7_empty_comments_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::from("# ARCH\n\n");
        for i in 1..=9 {
            if i == 7 {
                content.push_str(&format!(
                    "## {i}. Section {i}\n\n<!-- template comment -->\n\n"
                ));
            } else {
                content.push_str(&format!("## {i}. Section {i}\n\nContent.\n\n"));
            }
        }
        write_arch(dir.path(), &content);
        let results = check_architecture(&config_with_dir(dir.path()));
        let s7 = results
            .iter()
            .find(|r| r.name == "architecture-section-7")
            .unwrap();
        assert!(matches!(s7.status, CheckStatus::Fail(ref s) if s == "Section 7 empty"));
    }

    #[test]
    fn test_fail_section_9_missing() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::from("# ARCH\n\n");
        for i in 1..=8 {
            content.push_str(&format!("## {i}. Section {i}\n\nContent.\n\n"));
        }
        write_arch(dir.path(), &content);
        let results = check_architecture(&config_with_dir(dir.path()));
        let s9 = results
            .iter()
            .find(|r| r.name == "architecture-section-9")
            .unwrap();
        assert!(matches!(s9.status, CheckStatus::Fail(ref s) if s == "Section 9 missing"));
    }

    #[test]
    fn test_fail_section_empty_whitespace_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::from("# ARCH\n\n");
        for i in 1..=9 {
            if i == 8 {
                content.push_str(&format!("## {i}. Section {i}\n\n   \n\n"));
            } else {
                content.push_str(&format!("## {i}. Section {i}\n\nContent.\n\n"));
            }
        }
        write_arch(dir.path(), &content);
        let results = check_architecture(&config_with_dir(dir.path()));
        let s8 = results
            .iter()
            .find(|r| r.name == "architecture-section-8")
            .unwrap();
        assert!(matches!(s8.status, CheckStatus::Fail(ref s) if s == "Section 8 empty"));
    }

    #[test]
    fn test_disabled_returns_pass() {
        let config = Config {
            architecture: crate::config::ArchitectureConfig {
                enabled: false,
                ..crate::config::ArchitectureConfig::default()
            },
            ..Config::default()
        };
        let results = check_architecture(&config);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass));
    }

    #[test]
    fn test_pass_custom_required_sections() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::from("# ARCH\n\n");
        for i in 1..=3 {
            content.push_str(&format!("## {i}. Section {i}\n\nContent.\n\n"));
        }
        write_arch(dir.path(), &content);
        let config = Config {
            docs_dir: dir.path().to_path_buf(),
            architecture: crate::config::ArchitectureConfig {
                required_sections: 3,
                required_non_empty: vec![1, 2],
                ..crate::config::ArchitectureConfig::default()
            },
            ..Config::default()
        };
        let results = check_architecture(&config);
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, CheckStatus::Pass))
        );
    }

    fn write_named(dir: &std::path::Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    fn three_sections() -> String {
        let mut s = String::from("# DOC\n\n");
        for i in 1..=3 {
            s.push_str(&format!("## {i}. Section {i}\n\nContent {i}.\n\n"));
        }
        s
    }

    #[test]
    fn test_doc_structure_empty_returns_no_results() {
        let config = Config::default();
        assert!(check_doc_structure(&config).is_empty());
    }

    #[test]
    fn test_doc_structure_single_entry_pass() {
        let dir = tempfile::tempdir().unwrap();
        write_named(dir.path(), "TEST_CASES.md", &three_sections());
        let config = Config {
            docs_dir: dir.path().to_path_buf(),
            doc_structure: vec![crate::config::DocStructureConfig {
                file: "TEST_CASES.md".into(),
                required_sections: 3,
                required_non_empty: vec![1, 2, 3],
            }],
            ..Config::default()
        };
        let results = check_doc_structure(&config);
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, CheckStatus::Pass)),
            "{results:?}"
        );
        assert!(
            results
                .iter()
                .all(|r| r.name.starts_with("doc-TEST_CASES.md"))
        );
    }

    #[test]
    fn test_doc_structure_multiple_entries_independent() {
        let dir = tempfile::tempdir().unwrap();
        write_named(dir.path(), "A.md", &three_sections());
        // B.md only has 2 sections — should fail count check
        let mut b = String::from("# B\n\n");
        for i in 1..=2 {
            b.push_str(&format!("## {i}. Section\n\nContent.\n\n"));
        }
        write_named(dir.path(), "B.md", &b);
        let config = Config {
            docs_dir: dir.path().to_path_buf(),
            doc_structure: vec![
                crate::config::DocStructureConfig {
                    file: "A.md".into(),
                    required_sections: 3,
                    required_non_empty: vec![],
                },
                crate::config::DocStructureConfig {
                    file: "B.md".into(),
                    required_sections: 3,
                    required_non_empty: vec![],
                },
            ],
            ..Config::default()
        };
        let results = check_doc_structure(&config);
        let a = results
            .iter()
            .find(|r| r.name == "doc-A.md-section-count")
            .unwrap();
        let b = results
            .iter()
            .find(|r| r.name == "doc-B.md-section-count")
            .unwrap();
        assert!(matches!(a.status, CheckStatus::Pass));
        assert!(matches!(b.status, CheckStatus::Fail(_)));
    }

    #[test]
    fn test_doc_structure_missing_file_fails() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            docs_dir: dir.path().to_path_buf(),
            doc_structure: vec![crate::config::DocStructureConfig {
                file: "NONEXISTENT.md".into(),
                required_sections: 1,
                required_non_empty: vec![],
            }],
            ..Config::default()
        };
        let results = check_doc_structure(&config);
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.starts_with("File not found"))
        );
        assert_eq!(results[0].name, "doc-NONEXISTENT.md");
    }

    #[test]
    fn test_doc_structure_does_not_collide_with_architecture() {
        // Both [architecture] and [[doc_structure]] target the same file —
        // results should be distinct (different name prefixes).
        let dir = tempfile::tempdir().unwrap();
        write_arch(dir.path(), &full_9_sections());
        let config = Config {
            docs_dir: dir.path().to_path_buf(),
            doc_structure: vec![crate::config::DocStructureConfig {
                file: "ARCHITECTURE.md".into(),
                required_sections: 9,
                required_non_empty: vec![],
            }],
            ..Config::default()
        };
        let arch_results = check_architecture(&config);
        let doc_results = check_doc_structure(&config);
        assert!(arch_results.iter().any(|r| r.name.starts_with("architecture")));
        assert!(
            doc_results
                .iter()
                .any(|r| r.name.starts_with("doc-ARCHITECTURE.md"))
        );
        assert!(
            !doc_results
                .iter()
                .any(|r| r.name.starts_with("architecture"))
        );
    }
}
