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
