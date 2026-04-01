use crate::checks::{CheckResult, CheckStatus};
use crate::config::Config;

pub fn check_architecture(config: &Config) -> Vec<CheckResult> {
    let path = config.docs_dir.join(&config.architecture);
    let name_prefix = "architecture";

    let content = match std::fs::read_to_string(&path) {
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

    // Collect section positions and numbers
    let mut sections: Vec<(usize, usize)> = Vec::new(); // (line_idx, section_number)
    for (i, line) in lines.iter().enumerate() {
        if let Some(caps) = re_section.captures(line)
            && let Ok(num) = caps[1].parse::<usize>()
        {
            sections.push((i, num));
        }
    }

    let mut results = Vec::new();

    // Check total section count
    let unique_sections: std::collections::HashSet<usize> =
        sections.iter().map(|(_, n)| *n).collect();
    let found = unique_sections.len();
    let required = config.required_sections;

    if found < required {
        results.push(CheckResult {
            name: format!("{name_prefix}-section-count"),
            status: CheckStatus::Fail(format!("Found {found}/{required} sections")),
        });
    } else {
        results.push(CheckResult {
            name: format!("{name_prefix}-section-count"),
            status: CheckStatus::Pass,
        });
    }

    // Check required non-empty sections
    for &section_num in &config.required_non_empty {
        let section_pos = sections.iter().find(|(_, n)| *n == section_num);

        match section_pos {
            None => {
                results.push(CheckResult {
                    name: format!("{name_prefix}-section-{section_num}"),
                    status: CheckStatus::Fail(format!("Section {section_num} missing")),
                });
            }
            Some(&(line_idx, _)) => {
                // Find next section heading
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn config_with_dir(dir: &std::path::Path) -> Config {
        Config {
            docs_dir: dir.to_path_buf(),
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
            s.push_str(&format!("## {i}. Section {i}\n\nContent for section {i}.\n\n"));
        }
        s
    }

    #[test]
    fn test_pass_full_9_sections() {
        let dir = tempfile::tempdir().unwrap();
        write_arch(dir.path(), &full_9_sections());
        let results = check_architecture(&config_with_dir(dir.path()));
        assert!(results.iter().all(|r| matches!(r.status, CheckStatus::Pass)));
    }

    #[test]
    fn test_fail_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let results = check_architecture(&config_with_dir(dir.path()));
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Fail(ref s) if s.starts_with("File not found")));
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
        let count_result = results.iter().find(|r| r.name == "architecture-section-count").unwrap();
        assert!(matches!(count_result.status, CheckStatus::Fail(ref s) if s == "Found 5/9 sections"));
    }

    #[test]
    fn test_fail_section_7_empty_comments_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut content = String::from("# ARCH\n\n");
        for i in 1..=9 {
            if i == 7 {
                content.push_str(&format!("## {i}. Section {i}\n\n<!-- template comment -->\n\n"));
            } else {
                content.push_str(&format!("## {i}. Section {i}\n\nContent.\n\n"));
            }
        }
        write_arch(dir.path(), &content);
        let results = check_architecture(&config_with_dir(dir.path()));
        let s7 = results.iter().find(|r| r.name == "architecture-section-7").unwrap();
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
        let s9 = results.iter().find(|r| r.name == "architecture-section-9").unwrap();
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
        let s8 = results.iter().find(|r| r.name == "architecture-section-8").unwrap();
        assert!(matches!(s8.status, CheckStatus::Fail(ref s) if s == "Section 8 empty"));
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
            required_sections: 3,
            required_non_empty: vec![1, 2],
            ..Config::default()
        };
        let results = check_architecture(&config);
        assert!(results.iter().all(|r| matches!(r.status, CheckStatus::Pass)));
    }
}
