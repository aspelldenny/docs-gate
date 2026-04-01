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
