use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::{json, Value};

fn spawn_server() -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_docs-gate"))
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn docs-gate serve")
}

fn send_jsonrpc(stdin: &mut impl Write, stdout: &mut impl BufRead, request: &Value) -> Value {
    let msg = serde_json::to_string(request).unwrap();
    writeln!(stdin, "{msg}").unwrap();
    stdin.flush().unwrap();

    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    serde_json::from_str(&line).expect("Failed to parse JSON-RPC response")
}

fn initialize_request() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    })
}

fn initialized_notification() -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    })
}

fn tools_list_request(id: u64) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/list",
        "params": {}
    })
}

fn call_tool_request(id: u64, name: &str, args: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "tools/call",
        "params": {
            "name": name,
            "arguments": args
        }
    })
}

fn setup_valid_docs(dir: &std::path::Path) {
    let docs_dir = dir.join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();

    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let changelog = format!("# CHANGELOG\n\n## [v1] Release — {today}\n- Added stuff\n");
    std::fs::write(docs_dir.join("CHANGELOG.md"), changelog).unwrap();

    let mut arch = String::from("# ARCHITECTURE\n\n");
    for i in 1..=9 {
        arch.push_str(&format!("## {i}. Section {i}\n\nContent for section {i}.\n\n"));
    }
    std::fs::write(docs_dir.join("ARCHITECTURE.md"), arch).unwrap();
}

fn setup_valid_discovery(dir: &std::path::Path) -> String {
    let file = dir.join("discovery.md");
    std::fs::write(
        &file,
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
    .unwrap();
    file.to_string_lossy().to_string()
}

struct McpSession {
    child: std::process::Child,
    stdin: std::io::BufWriter<std::process::ChildStdin>,
    stdout: BufReader<std::process::ChildStdout>,
}

impl McpSession {
    fn start() -> Self {
        let mut child = spawn_server();
        let stdin = std::io::BufWriter::new(child.stdin.take().unwrap());
        let stdout = BufReader::new(child.stdout.take().unwrap());
        McpSession {
            child,
            stdin,
            stdout,
        }
    }

    fn start_in_dir(dir: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_docs-gate"))
            .arg("serve")
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn docs-gate serve");
        let stdin = std::io::BufWriter::new(child.stdin.take().unwrap());
        let stdout = BufReader::new(child.stdout.take().unwrap());
        McpSession {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, request: &Value) -> Value {
        send_jsonrpc(&mut self.stdin, &mut self.stdout, request)
    }

    fn send_notification(&mut self, notification: &Value) {
        let msg = serde_json::to_string(notification).unwrap();
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn initialize(&mut self) -> Value {
        let resp = self.send(&initialize_request());
        self.send_notification(&initialized_notification());
        resp
    }

    fn shutdown(mut self) {
        drop(self.stdin);
        let _ = self.child.wait_timeout(Duration::from_secs(5));
    }
}

trait WaitTimeout {
    fn wait_timeout(&mut self, dur: Duration) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl WaitTimeout for std::process::Child {
    fn wait_timeout(
        &mut self,
        dur: Duration,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let start = std::time::Instant::now();
        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None if start.elapsed() >= dur => return Ok(None),
                None => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }
}

// === Tests ===

#[test]
fn test_mcp_initialize_handshake() {
    let mut session = McpSession::start();
    let resp = session.initialize();

    assert_eq!(resp["jsonrpc"], "2.0");
    assert_eq!(resp["id"], 1);
    let result = &resp["result"];
    assert_eq!(result["serverInfo"]["name"], "docs-gate");
    assert!(result["capabilities"]["tools"].is_object());

    session.shutdown();
}

#[test]
fn test_mcp_tools_list() {
    let mut session = McpSession::start();
    session.initialize();

    let resp = session.send(&tools_list_request(2));
    let tools = resp["result"]["tools"].as_array().expect("tools should be array");
    assert_eq!(tools.len(), 4);

    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"check_changelog"));
    assert!(names.contains(&"check_architecture"));
    assert!(names.contains(&"check_discovery"));
    assert!(names.contains(&"check_all"));

    session.shutdown();
}

#[test]
fn test_mcp_check_changelog_pass() {
    let dir = tempfile::tempdir().unwrap();
    setup_valid_docs(dir.path());

    let mut session = McpSession::start_in_dir(dir.path());
    session.initialize();

    let resp = session.send(&call_tool_request(
        3,
        "check_changelog",
        json!({}),
    ));

    let content = &resp["result"]["content"];
    let text = content[0]["text"].as_str().unwrap();
    let results: Vec<Value> = serde_json::from_str(text).unwrap();
    assert_eq!(results[0]["status"], "Pass");

    session.shutdown();
}

#[test]
fn test_mcp_check_changelog_fail() {
    let dir = tempfile::tempdir().unwrap();
    // No docs dir → changelog not found

    let mut session = McpSession::start_in_dir(dir.path());
    session.initialize();

    let resp = session.send(&call_tool_request(
        3,
        "check_changelog",
        json!({}),
    ));

    let content = &resp["result"]["content"];
    let text = content[0]["text"].as_str().unwrap();
    let results: Vec<Value> = serde_json::from_str(text).unwrap();
    assert!(results[0]["status"].is_object()); // Fail(String) serializes as object

    session.shutdown();
}

#[test]
fn test_mcp_check_architecture_pass() {
    let dir = tempfile::tempdir().unwrap();
    setup_valid_docs(dir.path());

    let mut session = McpSession::start_in_dir(dir.path());
    session.initialize();

    let resp = session.send(&call_tool_request(
        4,
        "check_architecture",
        json!({}),
    ));

    let content = &resp["result"]["content"];
    let text = content[0]["text"].as_str().unwrap();
    let results: Vec<Value> = serde_json::from_str(text).unwrap();
    // All architecture checks should pass
    for result in &results {
        assert_eq!(result["status"], "Pass", "Failed: {}", result["name"]);
    }

    session.shutdown();
}

#[test]
fn test_mcp_check_discovery_pass() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = setup_valid_discovery(dir.path());

    let mut session = McpSession::start_in_dir(dir.path());
    session.initialize();

    let resp = session.send(&call_tool_request(
        5,
        "check_discovery",
        json!({ "file_path": file_path }),
    ));

    let content = &resp["result"]["content"];
    let text = content[0]["text"].as_str().unwrap();
    let results: Vec<Value> = serde_json::from_str(text).unwrap();
    for result in &results {
        assert_eq!(result["status"], "Pass", "Failed: {}", result["name"]);
    }

    session.shutdown();
}

#[test]
fn test_mcp_check_discovery_missing_path() {
    let mut session = McpSession::start();
    session.initialize();

    // Call check_discovery without file_path — should error
    let resp = session.send(&call_tool_request(
        6,
        "check_discovery",
        json!({}),
    ));

    // Should be an error response (missing required parameter)
    assert!(
        resp.get("error").is_some() || {
            // Or the tool might handle it gracefully with Fail result
            let content = &resp["result"]["content"];
            content[0]["text"].as_str().map_or(false, |t| t.contains("Fail"))
        },
        "Expected error or Fail for missing file_path: {resp}"
    );

    session.shutdown();
}

#[test]
fn test_mcp_check_all() {
    let dir = tempfile::tempdir().unwrap();
    setup_valid_docs(dir.path());

    // Create ticket dir with valid ticket
    let ticket_dir = dir.path().join("docs").join("ticket");
    std::fs::create_dir_all(&ticket_dir).unwrap();
    std::fs::write(
        ticket_dir.join("PHIEU-1.md"),
        "# Phiếu\n\n**Type:** `mutating`\n",
    )
    .unwrap();

    let mut session = McpSession::start_in_dir(dir.path());
    session.initialize();

    let resp = session.send(&call_tool_request(
        7,
        "check_all",
        json!({}),
    ));

    let content = &resp["result"]["content"];
    let text = content[0]["text"].as_str().unwrap();
    let results: Vec<Value> = serde_json::from_str(text).unwrap();
    // Should have changelog + architecture + ticket checks
    assert!(results.len() >= 3, "Expected at least 3 results, got {}", results.len());

    session.shutdown();
}

#[test]
fn test_mcp_graceful_shutdown() {
    let mut session = McpSession::start();
    session.initialize();

    // Close stdin to signal disconnect
    drop(session.stdin);

    // Process should exit cleanly within 5 seconds
    let status = session
        .child
        .wait_timeout(Duration::from_secs(5))
        .expect("wait_timeout failed");
    assert!(status.is_some(), "Server should exit after client disconnect");
}
