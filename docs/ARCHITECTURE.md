# ARCHITECTURE.md — docs-gate

> 9 sections bắt buộc. Section 7, 8, 9 PHẢI có.
> Kiến trúc sư đọc file này để viết phiếu. Sai ở đây = phiếu sai.

---

## 1. Module Map

```
src/
├── main.rs          — Entry point: CLI arg parsing + orchestration (async)
├── config.rs        — Load .docs-gate.toml, defaults, validation
├── watch.rs         — Watch mode: file watcher + re-run loop
├── mcp/
│   ├── mod.rs       — MCP module entry point
│   ├── server.rs    — DocsGateServer + ServerHandler impl
│   └── tools.rs     — Tool parameter types + config resolution
├── checks/
│   ├── mod.rs       — CheckResult type + run_all_checks() + run_all_checks_extended()
│   ├── changelog.rs — Check CHANGELOG.md has recent entry
│   ├── architecture.rs — Check ARCHITECTURE.md 9 sections + non-empty 7,8,9
│   ├── discovery.rs — Check Discovery Report format (4 required sections)
│   └── ticket.rs    — Check ticket files have valid Type declarations
└── output.rs        — Format results: human-readable + exit code
```

---

## 2. Public API

```rust
// main.rs
async fn main() -> ExitCode  // #[tokio::main(flavor = "current_thread")]
  // Parse CLI args (clap) → load config → route by flags → exit
  // Subcommand: check-discovery <file>
  // Flags: --all (includes ticket checks), --watch (watch mode)

// watch.rs
pub async fn run_watch(config: &Config, extended: bool) -> ExitCode
  // Run checks, setup notify watcher, debounce 500ms, re-run on changes, Ctrl+C exit

// mcp/server.rs
pub struct DocsGateServer { config: Config, tool_router: ToolRouter<Self> }
impl DocsGateServer { pub fn new(config: Config) -> Self }
impl ServerHandler for DocsGateServer { fn get_info(&self) -> ServerInfo }
// Tools (via #[tool_router] + #[tool] macros):
//   check_changelog(DocsDirParam) -> Json<Vec<CheckResult>>
//   check_architecture(DocsDirParam) -> Json<Vec<CheckResult>>
//   check_discovery(FilePathParam) -> Json<Vec<CheckResult>>
//   check_all(DocsDirParam) -> Json<Vec<CheckResult>>

// mcp/tools.rs
pub struct DocsDirParam { pub docs_dir: Option<String> }
pub struct FilePathParam { pub file_path: String }
pub fn resolve_config(base: &Config, docs_dir: Option<String>) -> Config

// config.rs
pub struct Config {
    pub docs_dir: PathBuf,          // default: "docs"
    pub changelog: String,          // default: "CHANGELOG.md"
    pub architecture: String,       // default: "ARCHITECTURE.md"
    pub required_sections: usize,   // default: 9
    pub required_non_empty: Vec<usize>, // default: [7, 8, 9]
    pub changelog_max_age_days: u32,    // default: 1
    pub ticket: TicketConfig,       // nested [ticket] section
}
pub struct TicketConfig {
    pub ticket_dir: PathBuf,        // default: "docs/ticket"
    pub valid_types: Vec<String>,   // default: ["read-only", "mutating", "destructive"]
    pub exclude_files: Vec<String>, // default: ["TEMPLATE.md"]
}
pub fn load_config(path: Option<&Path>) -> Config

// checks/mod.rs
pub enum CheckStatus { Pass, Fail(String), Warn(String) }
pub struct CheckResult { pub name: String, pub status: CheckStatus }
pub fn run_all_checks(config: &Config) -> Vec<CheckResult>
pub fn run_all_checks_extended(config: &Config) -> Vec<CheckResult>

// checks/changelog.rs
pub fn check_changelog(config: &Config) -> CheckResult

// checks/architecture.rs
pub fn check_architecture(config: &Config) -> Vec<CheckResult>

// checks/discovery.rs
pub fn check_discovery(file_path: &Path) -> Vec<CheckResult>

// checks/ticket.rs
pub fn check_tickets(config: &Config) -> Vec<CheckResult>

// output.rs
pub fn format_results(results: &[CheckResult], verbose: bool) -> String
pub fn exit_code(results: &[CheckResult]) -> ExitCode
```

---

## 3. Data Flow

```
CLI args (clap)
  → load_config(.docs-gate.toml hoặc defaults)
  → Route by command/flags:
    ├─ --watch + subcommand → error exit 2
    ├─ serve:
    │   → DocsGateServer::new(config)
    │   → server.serve(rmcp::transport::stdio())
    │   → MCP JSON-RPC loop on stdin/stdout
    │   → Client calls tool → route to check function → return JSON CheckResult(s)
    │   → Client disconnect → clean exit
    ├─ --watch:
    │   → run_watch(config, extended)
    │     → Run checks lần đầu
    │     → Setup notify watcher (docs_dir + ticket_dir nếu --all)
    │     → Loop: recv event → debounce 500ms → re-run checks
    │     → Ctrl+C → exit với last check exit code
    ├─ check-discovery <file>:
    │   → check_discovery(file) → format → exit
    ├─ --all:
    │   → run_all_checks_extended(config)
    │     ├─ check_changelog()
    │     ├─ check_architecture()
    │     └─ check_tickets() → scan ticket_dir/*.md, skip exclude_files
    │   → format → exit
    └─ (default):
        → run_all_checks(config)
          ├─ check_changelog()
          │   → Read file → find entries → check date/content
          └─ check_architecture()
              → Read file → parse ## headings → count sections
              → Check sections 7,8,9 not empty
        → format_results() → stdout
        → exit_code() → 0 (all pass) hoặc 1 (any fail)
```

---

## 4. Config

File: `.docs-gate.toml` (project root, optional)

| Option | Type | Default | Mô tả |
|--------|------|---------|-------|
| `docs_dir` | String | `"docs"` | Thư mục chứa docs |
| `changelog` | String | `"CHANGELOG.md"` | Tên file changelog |
| `architecture` | String | `"ARCHITECTURE.md"` | Tên file architecture |
| `required_sections` | u32 | `9` | Số sections bắt buộc |
| `required_non_empty` | Array<u32> | `[7, 8, 9]` | Sections phải có content |
| `changelog_max_age_days` | u32 | `1` | Changelog entry tối đa bao nhiêu ngày |
| `[ticket].ticket_dir` | String | `"docs/ticket"` | Thư mục chứa phiếu |
| `[ticket].valid_types` | Array<String> | `["read-only", "mutating", "destructive"]` | Các type hợp lệ |
| `[ticket].exclude_files` | Array<String> | `["TEMPLATE.md"]` | Files bỏ qua khi scan |

Không có file config → dùng defaults. Mọi option đều optional.
Nested `[ticket]` section: flat keys cũ giữ nguyên, keys mới nằm dưới `[ticket]`.

---

## 5. Dependencies

| Package | Version | Dùng cho |
|---------|---------|----------|
| clap | 4.x | CLI argument parsing (derive feature) |
| serde | 1.x | Deserialize config |
| toml | 0.8.x | Parse .docs-gate.toml |
| regex | 1.x | Parse markdown headings |
| chrono | 0.4.x | Date parsing cho changelog age check |
| tokio | 1.x | Async runtime (current_thread) cho watch mode + signal handling |
| notify | 8.x | Filesystem watcher cho watch mode |
| rmcp | 0.8.x | MCP server SDK (features: server, transport-io, macros) |
| schemars | 1.x | JSON Schema generation cho tool parameters |
| serde_json | 1.x | JSON serialization cho MCP tool responses |

---

## 6. Error Handling

| Lỗi | Xử lý |
|------|--------|
| File không tồn tại | CheckStatus::Fail("File not found: {path}") |
| File không đọc được | CheckStatus::Fail("Cannot read: {path}: {error}") |
| Config parse error | Stderr warning, dùng defaults |
| Invalid regex | Panic (bug, không phải user error) |
| Section heading parse fail | Skip section, count as missing |
| Ticket dir không tồn tại | CheckStatus::Warn("Ticket directory not found: {path}") |
| Ticket file thiếu Type | CheckStatus::Fail("{filename}: missing Type declaration") |
| Ticket type không hợp lệ | CheckStatus::Fail("{filename}: invalid type '{value}'") |

---

## 7. Implementation Notes ⛔ BẮT BUỘC

### changelog.rs
- **Algorithm:** Read file → split by `## ` headings → check first entry có date trong N ngày
- **Date detection:** Regex `\d{4}-\d{2}-\d{2}` trong heading hoặc first line of entry
- **"Updated" check:** Entry phải có content (>= 1 non-empty line sau heading)
- **Complexity:** O(n) single pass, n = file size
- **KHÔNG handle:** Changelog format khác (keep-a-changelog, conventional-changelog) — chỉ detect `## ` + date

### architecture.rs
- **Algorithm:** Read file → regex match `^## \d+\.` → collect section numbers → check count + non-empty
- **"Non-empty" check:** Section content (between 2 headings) có >= 1 non-comment, non-whitespace line
- **Comment detection:** Lines starting with `<!--` ignored (template comments)
- **Complexity:** O(n) single pass
- **KHÔNG handle:** Nested subsections (### ) — chỉ count top-level ## sections

### discovery.rs
- **Algorithm:** Read file → find `## Discovery Report` heading → check 4 required `### ` sub-headings → check each has content
- **Content check:** Between heading and next heading, >= 1 non-empty, non-comment line
- **Comment detection:** Lines starting with `<!--` ignored
- **Complexity:** O(n) single pass
- **KHÔNG handle:** Discovery Report embedded in larger docs with conflicting headings

### ticket.rs
- **Algorithm:** Read ticket_dir → filter *.md, exclude exclude_files → for each: regex match `\*\*Type:\*\*\s*` `` ` `` `([^` `` ` `` `]+)` → validate against valid_types
- **Regex:** `\*\*Type:\*\*\s*` followed by backtick-wrapped value
- **Complexity:** O(n*m) where n = number of files, m = avg file size
- **KHÔNG handle:** Multiple Type declarations with different values in same file — takes first valid match

### config.rs
- **Algorithm:** Check `.docs-gate.toml` exists → parse → merge with defaults
- **Merge strategy:** Config values override defaults, missing values use defaults
- **Data structure:** Flat struct for existing keys + nested TicketConfig struct for `[ticket]` section
- **Backward compat:** Flat keys unchanged; `[ticket]` section optional with `#[serde(default)]`

### mcp/server.rs + mcp/tools.rs
- **Architecture:** DocsGateServer struct holds Config + ToolRouter. #[tool_router] macro generates routing from tool name to handler method.
- **Tools:** 4 tools exposed via #[tool] macro. Each tool calls existing sync check functions directly — no logic duplication.
- **Parameter resolution:** DocsDirParam.docs_dir overrides Config.docs_dir if provided, otherwise uses config default. Config object is cloned per-call, not modified.
- **Return format:** All tools return Json<Vec<CheckResult>> — rmcp serializes to JSON text content in MCP response.
- **Transport:** stdio only. stdout = JSON-RPC channel. All logs/errors → stderr.
- **Lifecycle:** Config loaded once at startup. Server runs until client disconnect.
- **KHÔNG handle:** Hot-reload config. HTTP/SSE transport. Resources or prompts (tools only).

### watch.rs
- **Algorithm:** Run checks once → setup notify RecommendedWatcher on docs_dir (+ ticket_dir if extended) → event loop with tokio::select! between file events and Ctrl+C signal
- **Debounce:** 500ms hardcoded. On event recv, sleep 500ms then drain channel before re-running.
- **Terminal:** Clear screen (`\x1B[2J\x1B[H`) + print `[HH:MM:SS] Running checks...` before each run
- **Signal handling:** tokio::signal::ctrl_c() for clean exit
- **Exit code:** Last check run's exit code returned on Ctrl+C
- **Complexity:** O(1) per event (debounce), O(n) per check run
- **KHÔNG handle:** Debounce is not configurable. notify watcher errors on ticket_dir are warnings (dir may not exist yet).

### output.rs
- **Format:** `✅ PASS: {name}` hoặc `❌ FAIL: {name} — {reason}`
- **Exit code:** 0 nếu zero Fail, 1 nếu >= 1 Fail. Warn không ảnh hưởng exit code.

---

## 8. Runtime Behavior ⛔ BẮT BUỘC

- **Process model:** Foreground, async (tokio current_thread runtime)
- **Output:** stdout cho results, stderr cho warnings/errors
- **Exit codes:** 0 = all pass, 1 = any fail, 2 = config/usage error
- **Default mode:** Parse args → load config → run checks → exit. Nhanh, < 1 giây.
- **Watch mode (--watch):** Long-running process. Run checks → watch files → re-run on changes → Ctrl+C to exit.
  - Watcher: notify RecommendedWatcher (cross-platform) on docs_dir, + ticket_dir if --all
  - Debounce: 500ms hardcoded
  - Terminal: clear + timestamp before each re-run
  - Exit code: last check run's exit code
- **Serve mode (serve):** Long-running MCP server on stdio. JSON-RPC on stdin/stdout. Logs on stderr.
  - Config loaded once at startup, no hot-reload
  - Server exits when client disconnects
  - serve + --watch → error exit 2
- **Signal handling:** tokio::signal::ctrl_c() cho watch mode clean exit
- **File I/O:** Read-only. KHÔNG write bất kỳ file nào.

---

## 9. Known Constraints ⛔ BẮT BUỘC

### Markdown parsing đơn giản
- **Modules liên quan:** checks/architecture.rs
- **Vấn đề:** Dùng regex, không phải markdown parser. Heading trong code block bị count nhầm.
- **Xử lý hiện tại:** Chấp nhận. Edge case hiếm. Full parser quá nặng cho scope này.

### Changelog date format cứng
- **Modules liên quan:** checks/changelog.rs
- **Vấn đề:** Chỉ detect YYYY-MM-DD. Format khác (ISO 8601 variants) bị miss.
- **Xử lý hiện tại:** Document rõ. User cần dùng format này.

### Discovery Report heading exact match
- **Modules liên quan:** checks/discovery.rs
- **Vấn đề:** Headings phải exact match (bao gồm dấu tiếng Việt). Typo hoặc format khác bị miss.
- **Xử lý hiện tại:** Document rõ. Format cố định theo CLAUDE.md template.

### Ticket type regex cứng
- **Modules liên quan:** checks/ticket.rs
- **Vấn đề:** Chỉ detect `**Type:** \`value\`` format. Markdown bold + backtick. Format khác bị miss.
- **Xử lý hiện tại:** Document rõ. Đây là format chuẩn trong hệ thống thợ-thầu.

### Config file location cố định
- **Modules liên quan:** config.rs
- **Vấn đề:** Chỉ tìm .docs-gate.toml ở CWD. Không walk up directory tree.
- **Xử lý hiện tại:** Chấp nhận cho Phase 1. Thêm --config flag nếu cần.

### MCP server stdio only
- **Modules liên quan:** mcp/server.rs
- **Vấn đề:** Chỉ hỗ trợ stdio transport. Không SSE, không HTTP.
- **Xử lý hiện tại:** Đủ cho Claude Desktop/Code integration. Network transport ngoài scope.

### MCP server no config hot-reload
- **Modules liên quan:** mcp/server.rs
- **Vấn đề:** Config loaded 1 lần lúc startup. Thay đổi .docs-gate.toml cần restart server.
- **Xử lý hiện tại:** Chấp nhận. MCP clients sẽ restart server khi cần.
