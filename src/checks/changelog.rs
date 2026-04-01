use crate::checks::{CheckResult, CheckStatus};
use crate::config::Config;

pub fn check_changelog(config: &Config) -> CheckResult {
    let path = config.docs_dir.join(&config.changelog);
    let name = String::from("changelog");

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("File not found: {}", path.display())),
            };
        }
    };

    if content.trim().is_empty() {
        return CheckResult {
            name,
            status: CheckStatus::Fail(String::from("No entries found")),
        };
    }

    let re_heading = regex::Regex::new(r"^## ").unwrap();
    let re_date = regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut first_entry_heading: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        if re_heading.is_match(line) {
            first_entry_heading = Some(i);
            break;
        }
    }

    let heading_idx = match first_entry_heading {
        Some(i) => i,
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(String::from("No entries found")),
            };
        }
    };

    // Find date in heading or first content line
    let heading_line = lines[heading_idx];
    let date_str = re_date
        .find(heading_line)
        .or_else(|| lines.get(heading_idx + 1).and_then(|l| re_date.find(l)));

    let date_str = match date_str {
        Some(m) => m.as_str(),
        None => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(String::from("No date found in latest entry")),
            };
        }
    };

    // Check date age
    let entry_date = match chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return CheckResult {
                name,
                status: CheckStatus::Fail(format!("Invalid date: {date_str}")),
            };
        }
    };

    let today = chrono::Local::now().date_naive();
    let age = (today - entry_date).num_days();

    if age > i64::from(config.changelog_max_age_days) {
        return CheckResult {
            name,
            status: CheckStatus::Fail(format!("Last entry too old: {date_str}")),
        };
    }

    // Check entry has content
    let next_heading = lines
        .iter()
        .enumerate()
        .skip(heading_idx + 1)
        .find(|(_, l)| re_heading.is_match(l))
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    let has_content = lines[heading_idx + 1..next_heading].iter().any(|l| {
        let trimmed = l.trim();
        !trimmed.is_empty() && !trimmed.starts_with("<!--")
    });

    if !has_content {
        return CheckResult {
            name,
            status: CheckStatus::Fail(String::from("Entry empty")),
        };
    }

    CheckResult {
        name,
        status: CheckStatus::Pass,
    }
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

    fn write_changelog(dir: &std::path::Path, content: &str) {
        let path = dir.join("CHANGELOG.md");
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }

    fn today_str() -> String {
        chrono::Local::now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string()
    }

    fn days_ago_str(days: i64) -> String {
        let date = chrono::Local::now().date_naive() - chrono::Duration::days(days);
        date.format("%Y-%m-%d").to_string()
    }

    #[test]
    fn test_pass_entry_today() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(
            dir.path(),
            &format!(
                "# CHANGELOG\n\n## [v1] Release — {}\n- Added feature X\n",
                today_str()
            ),
        );
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(matches!(result.status, CheckStatus::Pass));
    }

    #[test]
    fn test_pass_entry_yesterday() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(
            dir.path(),
            &format!(
                "# CHANGELOG\n\n## [v1] Release — {}\n- Fixed bug\n",
                days_ago_str(1)
            ),
        );
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(matches!(result.status, CheckStatus::Pass));
    }

    #[test]
    fn test_fail_entry_too_old() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(
            dir.path(),
            &format!(
                "# CHANGELOG\n\n## [v1] Release — {}\n- Old stuff\n",
                days_ago_str(3)
            ),
        );
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(
            matches!(result.status, CheckStatus::Fail(ref s) if s.starts_with("Last entry too old"))
        );
    }

    #[test]
    fn test_fail_file_not_found() {
        let dir = tempfile::tempdir().unwrap();
        // No file created
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(
            matches!(result.status, CheckStatus::Fail(ref s) if s.starts_with("File not found"))
        );
    }

    #[test]
    fn test_fail_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(dir.path(), "");
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(matches!(result.status, CheckStatus::Fail(ref s) if s == "No entries found"));
    }

    #[test]
    fn test_fail_no_heading() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(
            dir.path(),
            "# CHANGELOG\n\nJust some text without entries.\n",
        );
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(matches!(result.status, CheckStatus::Fail(ref s) if s == "No entries found"));
    }

    #[test]
    fn test_fail_entry_empty_content() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(
            dir.path(),
            &format!(
                "# CHANGELOG\n\n## [v1] Release — {}\n\n## [v0] Old\n- stuff\n",
                today_str()
            ),
        );
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(matches!(result.status, CheckStatus::Fail(ref s) if s == "Entry empty"));
    }

    #[test]
    fn test_fail_no_date() {
        let dir = tempfile::tempdir().unwrap();
        write_changelog(
            dir.path(),
            "# CHANGELOG\n\n## [v1] Release without date\n- content\n",
        );
        let result = check_changelog(&config_with_dir(dir.path()));
        assert!(
            matches!(result.status, CheckStatus::Fail(ref s) if s == "No date found in latest entry")
        );
    }
}
