use std::process::ExitCode;

use crate::checks::{CheckResult, CheckStatus};

pub fn format_results(results: &[CheckResult], verbose: bool) -> String {
    let mut lines = Vec::new();

    for result in results {
        match &result.status {
            CheckStatus::Pass => {
                if verbose {
                    lines.push(format!("✅ PASS: {}", result.name));
                }
            }
            CheckStatus::Fail(reason) => {
                lines.push(format!("❌ FAIL: {} — {}", result.name, reason));
            }
            CheckStatus::Warn(reason) => {
                lines.push(format!("⚠️  WARN: {} — {}", result.name, reason));
            }
        }
    }

    // Always show summary
    let total = results.len();
    let passed = results
        .iter()
        .filter(|r| matches!(r.status, CheckStatus::Pass))
        .count();
    let failed = results
        .iter()
        .filter(|r| matches!(r.status, CheckStatus::Fail(_)))
        .count();

    if failed == 0 {
        lines.push(format!("\n✅ All checks passed ({total}/{total})"));
    } else {
        lines.push(format!("\n❌ {failed} check(s) failed ({passed}/{total} passed)"));
    }

    lines.join("\n")
}

pub fn exit_code(results: &[CheckResult]) -> ExitCode {
    let has_fail = results
        .iter()
        .any(|r| matches!(r.status, CheckStatus::Fail(_)));

    if has_fail {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pass_result(name: &str) -> CheckResult {
        CheckResult { name: String::from(name), status: CheckStatus::Pass }
    }

    fn fail_result(name: &str, reason: &str) -> CheckResult {
        CheckResult { name: String::from(name), status: CheckStatus::Fail(String::from(reason)) }
    }

    #[test]
    fn test_all_pass_verbose() {
        let results = vec![pass_result("changelog"), pass_result("architecture")];
        let output = format_results(&results, true);
        assert!(output.contains("✅ PASS: changelog"));
        assert!(output.contains("✅ PASS: architecture"));
        assert!(output.contains("All checks passed (2/2)"));
    }

    #[test]
    fn test_all_pass_non_verbose() {
        let results = vec![pass_result("changelog"), pass_result("architecture")];
        let output = format_results(&results, false);
        assert!(!output.contains("PASS: changelog"));
        assert!(output.contains("All checks passed (2/2)"));
    }

    #[test]
    fn test_with_failure() {
        let results = vec![pass_result("changelog"), fail_result("architecture", "Section 7 empty")];
        let output = format_results(&results, false);
        assert!(output.contains("❌ FAIL: architecture — Section 7 empty"));
        assert!(output.contains("1 check(s) failed (1/2 passed)"));
    }

    #[test]
    fn test_exit_code_pass() {
        let results = vec![pass_result("a"), pass_result("b")];
        assert_eq!(exit_code(&results), ExitCode::from(0));
    }

    #[test]
    fn test_exit_code_fail() {
        let results = vec![pass_result("a"), fail_result("b", "broken")];
        assert_eq!(exit_code(&results), ExitCode::from(1));
    }
}
