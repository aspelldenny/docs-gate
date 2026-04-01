use std::io::Write;
use std::process::Command;

fn docs_gate_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docs-gate"))
}

fn create_fixture(dir: &std::path::Path, docs_dir: &str, files: &[(&str, &str)]) {
    let docs_path = dir.join(docs_dir);
    std::fs::create_dir_all(&docs_path).unwrap();
    for (name, content) in files {
        let path = docs_path.join(name);
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{content}").unwrap();
    }
}

fn today_str() -> String {
    chrono::Local::now().date_naive().format("%Y-%m-%d").to_string()
}

fn full_architecture() -> String {
    let mut s = String::from("# ARCHITECTURE\n\n");
    for i in 1..=9 {
        s.push_str(&format!("## {i}. Section {i}\n\nContent for section {i}.\n\n"));
    }
    s
}

#[test]
fn test_all_pass_exit_0() {
    let dir = tempfile::tempdir().unwrap();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n", today_str());
    create_fixture(dir.path(), "docs", &[
        ("CHANGELOG.md", &changelog),
        ("ARCHITECTURE.md", &full_architecture()),
    ]);

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .arg("--verbose")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Expected exit 0, got: {stdout}");
    assert!(stdout.contains("All checks passed"));
}

#[test]
fn test_missing_changelog_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    create_fixture(dir.path(), "docs", &[
        ("ARCHITECTURE.md", &full_architecture()),
    ]);

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "Expected exit 1, got: {stdout}");
    assert!(stdout.contains("FAIL: changelog"));
}

#[test]
fn test_missing_architecture_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n", today_str());
    create_fixture(dir.path(), "docs", &[
        ("CHANGELOG.md", &changelog),
    ]);

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "Expected exit 1");
    assert!(stdout.contains("FAIL: architecture"));
}

#[test]
fn test_verbose_shows_pass_results() {
    let dir = tempfile::tempdir().unwrap();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n", today_str());
    create_fixture(dir.path(), "docs", &[
        ("CHANGELOG.md", &changelog),
        ("ARCHITECTURE.md", &full_architecture()),
    ]);

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .arg("--verbose")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("✅ PASS: changelog"));
    assert!(stdout.contains("✅ PASS: architecture-section-count"));
}

#[test]
fn test_custom_config() {
    let dir = tempfile::tempdir().unwrap();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n", today_str());
    create_fixture(dir.path(), "mydocs", &[
        ("CHANGELOG.md", &changelog),
        ("ARCHITECTURE.md", &full_architecture()),
    ]);

    let config_path = dir.path().join(".docs-gate.toml");
    std::fs::write(&config_path, "docs_dir = \"mydocs\"\n").unwrap();

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success(), "Expected exit 0 with custom config");
}

fn valid_discovery() -> String {
    String::from(
        "## Discovery Report\n\
         ### Assumptions trong phiếu — ĐÚNG:\n\
         - All correct\n\
         ### Assumptions trong phiếu — SAI:\n\
         - Không có\n\
         ### Edge cases phát hiện thêm:\n\
         - Không có\n\
         ### Docs đã cập nhật:\n\
         - CHANGELOG.md\n",
    )
}

#[test]
fn test_check_discovery_pass() {
    let dir = tempfile::tempdir().unwrap();
    let report_path = dir.path().join("report.md");
    std::fs::write(&report_path, valid_discovery()).unwrap();

    let output = docs_gate_bin()
        .arg("check-discovery")
        .arg(&report_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Expected exit 0: {stdout}");
    assert!(stdout.contains("All checks passed"));
}

#[test]
fn test_check_discovery_fail_missing_section() {
    let dir = tempfile::tempdir().unwrap();
    let report_path = dir.path().join("report.md");
    std::fs::write(&report_path, "## Discovery Report\n### Assumptions trong phiếu — ĐÚNG:\n- Yes\n").unwrap();

    let output = docs_gate_bin()
        .arg("check-discovery")
        .arg(&report_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "Expected exit 1");
    assert!(stdout.contains("FAIL"));
}

#[test]
fn test_all_flag_includes_ticket_check() {
    let dir = tempfile::tempdir().unwrap();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n", today_str());
    create_fixture(dir.path(), "docs", &[
        ("CHANGELOG.md", &changelog),
        ("ARCHITECTURE.md", &full_architecture()),
    ]);
    // Create ticket dir with valid ticket
    let ticket_dir = dir.path().join("docs").join("ticket");
    std::fs::create_dir_all(&ticket_dir).unwrap();
    std::fs::write(ticket_dir.join("PHIEU-1.md"), "# Phiếu\n\n**Type:** `mutating`\n").unwrap();

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .arg("--all")
        .arg("--verbose")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Expected exit 0: {stdout}");
    assert!(stdout.contains("ticket"));
}

#[test]
fn test_watch_with_check_discovery_error() {
    let output = docs_gate_bin()
        .arg("--watch")
        .arg("check-discovery")
        .arg("some-file.md")
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr.contains("Watch mode not supported"));
}

#[test]
fn test_watch_sigint_clean_exit() {
    use std::time::Duration;

    let dir = tempfile::tempdir().unwrap();
    let changelog = format!(
        "# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n",
        today_str()
    );
    create_fixture(
        dir.path(),
        "docs",
        &[
            ("CHANGELOG.md", &changelog),
            ("ARCHITECTURE.md", &full_architecture()),
        ],
    );

    let child = std::process::Command::new(env!("CARGO_BIN_EXE_docs-gate"))
        .current_dir(dir.path())
        .arg("--watch")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    // Give it time to start and run initial checks
    std::thread::sleep(Duration::from_millis(500));

    // Send SIGINT (Ctrl+C equivalent)
    unsafe {
        libc::kill(child.id() as libc::pid_t, libc::SIGINT);
    }

    let output = child.wait_with_output().unwrap();
    // Should exit cleanly (exit 0 since checks pass)
    assert!(
        output.status.success(),
        "Expected clean exit after SIGINT, got: {:?}",
        output.status
    );
}

#[test]
fn test_default_no_ticket_check() {
    let dir = tempfile::tempdir().unwrap();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {}\n- Added stuff\n", today_str());
    create_fixture(dir.path(), "docs", &[
        ("CHANGELOG.md", &changelog),
        ("ARCHITECTURE.md", &full_architecture()),
    ]);
    // Create ticket dir with INVALID ticket (no Type)
    let ticket_dir = dir.path().join("docs").join("ticket");
    std::fs::create_dir_all(&ticket_dir).unwrap();
    std::fs::write(ticket_dir.join("BAD.md"), "# No type\n").unwrap();

    let output = docs_gate_bin()
        .current_dir(dir.path())
        .arg("--verbose")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Without --all, ticket check should NOT run, so bad ticket doesn't cause failure
    assert!(output.status.success(), "Expected exit 0 without --all: {stdout}");
    assert!(!stdout.contains("ticket"));
}
