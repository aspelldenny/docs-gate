use crate::checks::{CheckResult, CheckStatus};
use crate::config::Config;

pub fn check_tickets(config: &Config) -> Vec<CheckResult> {
    let ticket_dir = &config.ticket.ticket_dir;

    if !ticket_dir.exists() {
        return vec![CheckResult {
            name: String::from("ticket"),
            status: CheckStatus::Warn(format!(
                "Ticket directory not found: {}",
                ticket_dir.display()
            )),
        }];
    }

    let entries = match std::fs::read_dir(ticket_dir) {
        Ok(e) => e,
        Err(e) => {
            return vec![CheckResult {
                name: String::from("ticket"),
                status: CheckStatus::Fail(format!("Cannot read {}: {e}", ticket_dir.display())),
            }];
        }
    };

    let md_files: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".md") && !config.ticket.exclude_files.contains(&name)
        })
        .collect();

    if md_files.is_empty() {
        return vec![CheckResult {
            name: String::from("ticket"),
            status: CheckStatus::Pass,
        }];
    }

    // If type_pattern is empty, skip type validation
    let type_re = if config.ticket.type_pattern.is_empty() {
        None
    } else {
        match regex::Regex::new(&config.ticket.type_pattern) {
            Ok(re) => Some(re),
            Err(e) => {
                return vec![CheckResult {
                    name: String::from("ticket"),
                    status: CheckStatus::Fail(format!("Invalid type_pattern regex: {e}")),
                }];
            }
        }
    };

    let mut results = Vec::new();

    for entry in &md_files {
        let filename = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                results.push(CheckResult {
                    name: format!("ticket-{filename}"),
                    status: CheckStatus::Fail(format!("Cannot read {filename}: {e}")),
                });
                continue;
            }
        };

        // No type_pattern → pass without type validation
        let Some(ref re) = type_re else {
            results.push(CheckResult {
                name: format!("ticket-{filename}"),
                status: CheckStatus::Pass,
            });
            continue;
        };

        let captures: Vec<_> = re.captures_iter(&content).collect();

        if captures.is_empty() {
            results.push(CheckResult {
                name: format!("ticket-{filename}"),
                status: CheckStatus::Fail(format!("{filename}: missing Type declaration")),
            });
            continue;
        }

        let mut file_ok = true;
        for cap in &captures {
            let type_value = &cap[1];
            if !config.ticket.valid_types.iter().any(|t| t == type_value) {
                let valid = config.ticket.valid_types.join(", ");
                results.push(CheckResult {
                    name: format!("ticket-{filename}"),
                    status: CheckStatus::Fail(format!(
                        "{filename}: invalid type '{type_value}', expected one of: {valid}"
                    )),
                });
                file_ok = false;
                break;
            }
        }

        if file_ok {
            results.push(CheckResult {
                name: format!("ticket-{filename}"),
                status: CheckStatus::Pass,
            });
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
            ticket: crate::config::TicketConfig {
                ticket_dir: dir.to_path_buf(),
                ..crate::config::TicketConfig::default()
            },
            ..Config::default()
        }
    }

    fn write_ticket(dir: &std::path::Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    #[test]
    fn test_pass_all_valid() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(
            dir.path(),
            "PHIEU-1.md",
            "# Phiếu 1\n\n**Type:** `mutating`\n",
        );
        write_ticket(
            dir.path(),
            "PHIEU-2.md",
            "# Phiếu 2\n\n**Type:** `read-only`\n",
        );
        let results = check_tickets(&config_with_dir(dir.path()));
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, CheckStatus::Pass))
        );
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_fail_missing_type() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(dir.path(), "PHIEU-1.md", "# Phiếu 1\n\nNo type here\n");
        let results = check_tickets(&config_with_dir(dir.path()));
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("missing Type"))
        );
    }

    #[test]
    fn test_fail_invalid_type() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(
            dir.path(),
            "PHIEU-1.md",
            "# Phiếu 1\n\n**Type:** `urgent`\n",
        );
        let results = check_tickets(&config_with_dir(dir.path()));
        assert_eq!(results.len(), 1);
        assert!(
            matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("invalid type 'urgent'"))
        );
    }

    #[test]
    fn test_warn_dir_not_found() {
        let config = config_with_dir(std::path::Path::new("/tmp/nonexistent-ticket-dir"));
        let results = check_tickets(&config);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Warn(_)));
    }

    #[test]
    fn test_pass_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let results = check_tickets(&config_with_dir(dir.path()));
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass));
    }

    #[test]
    fn test_skip_template() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(dir.path(), "TEMPLATE.md", "# Template\n\nNo type here\n");
        write_ticket(
            dir.path(),
            "PHIEU-1.md",
            "# Phiếu 1\n\n**Type:** `mutating`\n",
        );
        let results = check_tickets(&config_with_dir(dir.path()));
        // TEMPLATE.md skipped, only PHIEU-1.md checked
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass));
    }

    #[test]
    fn test_pass_custom_type_pattern() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(
            dir.path(),
            "PHIEU-1.md",
            "# Phiếu 1\n\n**Loại:** `mutating`\n",
        );
        let config = Config {
            ticket: crate::config::TicketConfig {
                ticket_dir: dir.path().to_path_buf(),
                type_pattern: r"\*\*Loại:\*\*\s*`([^`]+)`".to_string(),
                ..crate::config::TicketConfig::default()
            },
            ..Config::default()
        };
        let results = check_tickets(&config);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass));
    }

    #[test]
    fn test_pass_no_type_pattern_skips_validation() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(
            dir.path(),
            "PHIEU-1.md",
            "# Phiếu 1\n\nNo type field at all\n",
        );
        let config = Config {
            ticket: crate::config::TicketConfig {
                ticket_dir: dir.path().to_path_buf(),
                type_pattern: String::new(),
                ..crate::config::TicketConfig::default()
            },
            ..Config::default()
        };
        let results = check_tickets(&config);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Pass));
    }

    #[test]
    fn test_pass_destructive_type() {
        let dir = tempfile::tempdir().unwrap();
        write_ticket(
            dir.path(),
            "PHIEU-1.md",
            "# Phiếu 1\n\n**Type:** `destructive`\n",
        );
        let results = check_tickets(&config_with_dir(dir.path()));
        assert!(
            results
                .iter()
                .all(|r| matches!(r.status, CheckStatus::Pass))
        );
    }
}
