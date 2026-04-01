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
