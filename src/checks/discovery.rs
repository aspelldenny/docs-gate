use std::path::Path;

use crate::checks::{CheckResult, CheckStatus};

const MAIN_HEADING: &str = "## Discovery Report";
const REQUIRED_SECTIONS: &[&str] = &[
    "### Assumptions trong phiếu — ĐÚNG:",
    "### Assumptions trong phiếu — SAI:",
    "### Edge cases phát hiện thêm:",
    "### Docs đã cập nhật:",
];

pub fn check_discovery(file_path: &Path) -> Vec<CheckResult> {
    let name_prefix = "discovery";

    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => {
            return vec![CheckResult {
                name: String::from(name_prefix),
                status: CheckStatus::Fail(format!("File not found: {}", file_path.display())),
            }];
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    // Check main heading exists
    let has_main = lines.iter().any(|l| l.trim() == MAIN_HEADING);
    if !has_main {
        return vec![CheckResult {
            name: String::from(name_prefix),
            status: CheckStatus::Fail(format!("Missing '{MAIN_HEADING}' heading")),
        }];
    }

    let mut results = Vec::new();

    for &section in REQUIRED_SECTIONS {
        let section_idx = lines.iter().position(|l| l.trim() == section);

        match section_idx {
            None => {
                results.push(CheckResult {
                    name: format!("{name_prefix}-section"),
                    status: CheckStatus::Fail(format!("Missing section: '{section}'")),
                });
            }
            Some(idx) => {
                // Find next heading (### or ##)
                let next_heading = lines
                    .iter()
                    .enumerate()
                    .skip(idx + 1)
                    .find(|(_, l)| {
                        let trimmed = l.trim();
                        trimmed.starts_with("### ") || trimmed.starts_with("## ")
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(lines.len());

                let has_content = lines[idx + 1..next_heading].iter().any(|l| {
                    let trimmed = l.trim();
                    !trimmed.is_empty() && !trimmed.starts_with("<!--")
                });

                if has_content {
                    results.push(CheckResult {
                        name: format!("{name_prefix}-section"),
                        status: CheckStatus::Pass,
                    });
                } else {
                    results.push(CheckResult {
                        name: format!("{name_prefix}-section"),
                        status: CheckStatus::Fail(format!("Section empty: '{section}'")),
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

    fn write_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "{content}").unwrap();
        path
    }

    fn valid_discovery() -> String {
        String::from(
            "## Discovery Report\n\
             ### Assumptions trong phiếu — ĐÚNG:\n\
             - Config loads correctly\n\
             ### Assumptions trong phiếu — SAI:\n\
             - Không có\n\
             ### Edge cases phát hiện thêm:\n\
             - Không có\n\
             ### Docs đã cập nhật:\n\
             - CHANGELOG.md\n",
        )
    }

    #[test]
    fn test_pass_valid_discovery() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "report.md", &valid_discovery());
        let results = check_discovery(&path);
        assert!(results.iter().all(|r| matches!(r.status, CheckStatus::Pass)));
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_fail_file_not_found() {
        let results = check_discovery(Path::new("/tmp/nonexistent-discovery.md"));
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Fail(ref s) if s.starts_with("File not found")));
    }

    #[test]
    fn test_fail_missing_main_heading() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_file(dir.path(), "report.md", "# Some other doc\n- stuff\n");
        let results = check_discovery(&path);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0].status, CheckStatus::Fail(ref s) if s.contains("Missing")));
    }

    #[test]
    fn test_fail_missing_section() {
        let dir = tempfile::tempdir().unwrap();
        let content = "## Discovery Report\n\
                        ### Assumptions trong phiếu — ĐÚNG:\n\
                        - Yes\n\
                        ### Assumptions trong phiếu — SAI:\n\
                        - No\n";
        let path = write_file(dir.path(), "report.md", content);
        let results = check_discovery(&path);
        let fails: Vec<_> = results
            .iter()
            .filter(|r| matches!(r.status, CheckStatus::Fail(_)))
            .collect();
        assert_eq!(fails.len(), 2); // missing 2 sections
    }

    #[test]
    fn test_fail_section_empty() {
        let dir = tempfile::tempdir().unwrap();
        let content = "## Discovery Report\n\
                        ### Assumptions trong phiếu — ĐÚNG:\n\
                        - Yes\n\
                        ### Assumptions trong phiếu — SAI:\n\
                        \n\
                        ### Edge cases phát hiện thêm:\n\
                        - None\n\
                        ### Docs đã cập nhật:\n\
                        - CHANGELOG.md\n";
        let path = write_file(dir.path(), "report.md", content);
        let results = check_discovery(&path);
        let fails: Vec<_> = results
            .iter()
            .filter(|r| matches!(r.status, CheckStatus::Fail(_)))
            .collect();
        assert_eq!(fails.len(), 1);
        assert!(matches!(fails[0].status, CheckStatus::Fail(ref s) if s.contains("SAI")));
    }

    #[test]
    fn test_pass_khong_co_counts_as_content() {
        let dir = tempfile::tempdir().unwrap();
        let content = "## Discovery Report\n\
                        ### Assumptions trong phiếu — ĐÚNG:\n\
                        - Đúng hết\n\
                        ### Assumptions trong phiếu — SAI:\n\
                        - Không có\n\
                        ### Edge cases phát hiện thêm:\n\
                        - Không có\n\
                        ### Docs đã cập nhật:\n\
                        - Không có\n";
        let path = write_file(dir.path(), "report.md", content);
        let results = check_discovery(&path);
        assert!(results.iter().all(|r| matches!(r.status, CheckStatus::Pass)));
    }

    #[test]
    fn test_fail_section_only_comments() {
        let dir = tempfile::tempdir().unwrap();
        let content = "## Discovery Report\n\
                        ### Assumptions trong phiếu — ĐÚNG:\n\
                        <!-- template -->\n\
                        ### Assumptions trong phiếu — SAI:\n\
                        - No\n\
                        ### Edge cases phát hiện thêm:\n\
                        - No\n\
                        ### Docs đã cập nhật:\n\
                        - No\n";
        let path = write_file(dir.path(), "report.md", content);
        let results = check_discovery(&path);
        let fails: Vec<_> = results
            .iter()
            .filter(|r| matches!(r.status, CheckStatus::Fail(_)))
            .collect();
        assert_eq!(fails.len(), 1);
        assert!(matches!(fails[0].status, CheckStatus::Fail(ref s) if s.contains("ĐÚNG")));
    }
}
