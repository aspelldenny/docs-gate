# ARCHITECTURE.md — docs-gate

> 9 sections bắt buộc. Section 7, 8, 9 PHẢI có.
> Kiến trúc sư đọc file này để viết phiếu. Sai ở đây = phiếu sai.

---

## 1. Module Map

```
src/
├── main.rs          — Entry point: CLI arg parsing + orchestration (async)
├── config.rs        — Load .docs-gate.toml, defaults, validation
├── init.rs          — Project scanner: detect docs structure → generate config
├── watch.rs         — Watch mode: file watcher + re-run loop
├── mcp/
│   ├── mod.rs       — MCP module entry point
│   ├── server.rs    — DocsGateServer + ServerHandler impl
│   └── tools.rs     — Tool parameter types + config resolution
├── checks/
│   ├── mod.rs       — CheckResult type + run_all_checks() + run_all_checks_extended()
│   ├── changelog.rs — Check CHANGELOG.md has recent entry
│   ├── architecture.rs — Check ARCHITECTURE.md + [[doc_structure]] entries
│   ├── count.rs     — Check [[count_check]] entries (doc number vs command output)
│   ├── cross_doc.rs — Check [[cross_doc]] entries (subset relationship between docs)
│   ├── discovery.rs — Check Discovery Report format (4 required sections)
│   ├── staged.rs    — Git-aware: changelog-staged + file-to-docs rules + staleness
│   └── ticket.rs    — Check ticket files (configurable type pattern)
└── output.rs        — Format results: human-readable + exit code
```

---

## 2. Public API

```rust
// main.rs
async fn main() -> ExitCode  // #[tokio::main(flavor = "current_thread")]
  // Parse CLI args (clap) → load config → route by flags → exit
  // Subcommand: check-discovery <file>, init, serve
  // Flags: --all (includes ticket checks), --watch (watch mode)

// watch.rs
pub async fn run_watch(config: &Config, extended: bool) -> ExitCode
  // Run checks, setup notify watcher, debounce 500ms, re-run on changes, Ctrl+C exit

// mcp/server.rs
pub struct DocsGateServer { config_path: Option<PathBuf>, tool_router: ToolRouter<Self> }
impl DocsGateServer {
    pub fn new(config_path: Option<PathBuf>) -> Self
    fn load_fresh_config(&self) -> Config  // reloads .docs-gate.toml each call
}
impl ServerHandler for DocsGateServer { fn get_info(&self) -> ServerInfo }
// Tools (via #[tool_router] + #[tool] macros):
//   check_changelog(DocsDirParam) -> Json<Vec<CheckResult>>
//   check_architecture(DocsDirParam) -> Json<Vec<CheckResult>>
//   check_discovery(FilePathParam) -> Json<Vec<CheckResult>>
//   check_staged(DocsDirParam) -> Json<Vec<CheckResult>>
//   check_all(DocsDirParam) -> Json<Vec<CheckResult>>

// mcp/tools.rs
pub struct DocsDirParam { pub docs_dir: Option<String> }
pub struct FilePathParam { pub file_path: String }
pub fn resolve_config(config: Config, docs_dir: Option<String>) -> Config  // takes by value (fresh per call)

// config.rs
pub struct Config {
    pub docs_dir: PathBuf,              // default: "docs"
    pub changelog: String,              // default: "CHANGELOG.md"
    pub changelog_max_age_days: u32,    // default: 1
    pub architecture: ArchitectureConfig,
    pub ticket: TicketConfig,
    pub changelog_staged: bool,         // default: true
    pub rules: Vec<RuleConfig>,         // default: []
    pub staleness: Vec<StalenessConfig>,// default: []
    pub doc_structure: Vec<DocStructureConfig>, // default: [] — extra structural checks
    pub count_check: Vec<CountCheckConfig>,      // default: [] — drift checks (doc vs command)
    pub cross_doc: Vec<CrossDocConfig>,          // default: [] — cross-doc subset checks
}
pub struct ArchitectureConfig {
    pub enabled: bool,                  // default: true — set false to skip
    pub file: String,                   // default: "ARCHITECTURE.md"
    pub required_sections: usize,       // default: 9
    pub required_non_empty: Vec<usize>, // default: [7, 8, 9]
}
pub struct DocStructureConfig {
    pub file: String,                   // path relative to docs_dir
    pub required_sections: usize,
    pub required_non_empty: Vec<usize>, // default: []
}
pub struct CountCheckConfig {
    pub file: String,                   // doc with hardcoded number
    pub doc_pattern: String,            // regex w/ capture group → number in doc
    pub command: String,                // shell command (sh -c on Unix, cmd /C on Windows)
    pub command_pattern: String,        // regex w/ capture group → number in command stdout
    pub description: String,            // default: "" — human label for error message
}
pub struct CrossDocConfig {
    pub source: String,                 // doc that declares the canonical set
    pub source_pattern: String,         // regex w/ capture group → values in source
    pub target: String,                 // doc that must contain those values
    pub target_pattern: String,         // regex w/ capture group → values in target
    pub description: String,            // default: ""
}
pub struct TicketConfig {
    pub ticket_dir: PathBuf,            // default: "docs/ticket"
    pub type_pattern: Option<String>,   // default: Some("\\*\\*Type:\\*\\*...")  — None = skip type check
    pub valid_types: Vec<String>,       // default: ["read-only", "mutating", "destructive"]
    pub exclude_files: Vec<String>,     // default: ["TEMPLATE.md"]
}
pub fn load_config(path: Option<&Path>) -> Config

// init.rs
pub fn scan_project(root: &Path) -> Config   // scan project → detect docs structure
pub fn write_config(root: &Path, config: &Config) -> io::Result<String>  // write .docs-gate.toml

// checks/mod.rs
pub enum CheckStatus { Pass, Fail(String), Warn(String) }
pub struct CheckResult { pub name: String, pub status: CheckStatus }
pub fn run_all_checks(config: &Config) -> Vec<CheckResult>
pub fn run_all_checks_extended(config: &Config) -> Vec<CheckResult>

// checks/changelog.rs
pub fn check_changelog(config: &Config) -> CheckResult

// checks/architecture.rs
pub fn check_doc_file(path: &Path, required_sections: usize, required_non_empty: &[usize], name_prefix: &str) -> Vec<CheckResult>
pub fn check_architecture(config: &Config) -> Vec<CheckResult>
pub fn check_doc_structure(config: &Config) -> Vec<CheckResult>  // iterates [[doc_structure]] entries

// checks/count.rs
pub fn check_counts(config: &Config) -> Vec<CheckResult>  // iterates [[count_check]] entries

// checks/cross_doc.rs
pub fn check_cross_doc(config: &Config) -> Vec<CheckResult>  // iterates [[cross_doc]] entries

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
    ├─ init:
    │   → scan_project(cwd)
    │     ├─ detect_docs_dir() → find docs/ or doc/ or documentation/
    │     ├─ detect_changelog() → find CHANGELOG.md variants
    │     ├─ detect_architecture() → find ARCHITECTURE.md, count sections
    │     └─ detect_tickets() → find ticket dir, detect type pattern + excludes
    │   → write_config() → .docs-gate.toml
    │   → print summary to stderr, config to stdout
    ├─ serve:
    │   → DocsGateServer::new(cli.config)  // pass config PATH, not parsed Config
    │   → server.serve(rmcp::transport::stdio())
    │   → MCP JSON-RPC loop on stdin/stdout
    │   → Client calls tool → load_fresh_config() reloads .docs-gate.toml from disk
    │   → resolve_config(fresh, override) → run checks → return JSON CheckResult(s)
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
          ├─ check_architecture()
          │   → Read file → parse ## headings → count sections
          │   → Check sections 7,8,9 not empty
          ├─ check_doc_structure()
          │   → For each [[doc_structure]] entry: same logic as architecture
          │   → No-op when array is empty (default)
          ├─ check_changelog_staged() + check_rules() + check_staleness()
          ├─ check_counts()
          │   → For each [[count_check]] entry: extract number from doc
          │   → Run command via sh -c (or cmd /C on Windows), parse stdout
          │   → Compare numbers; fail if mismatch
          │   → No-op when array is empty
          └─ check_cross_doc()
              → For each [[cross_doc]] entry: extract values from source + target
              → Verify target ⊇ source; fail with missing-values list (cap 5)
              → No-op when array is empty
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
| `changelog_max_age_days` | u32 | `1` | Changelog entry tối đa bao nhiêu ngày |
| `[architecture].enabled` | bool | `true` | Bật/tắt architecture check |
| `[architecture].file` | String | `"ARCHITECTURE.md"` | Tên file architecture |
| `[architecture].required_sections` | u32 | `9` | Số sections bắt buộc |
| `[architecture].required_non_empty` | Array<u32> | `[7, 8, 9]` | Sections phải có content |
| `[ticket].ticket_dir` | String | `"docs/ticket"` | Thư mục chứa phiếu |
| `[ticket].type_pattern` | String? | `"\\*\\*Type:\\*\\*..."` | Regex cho type field (null = skip) |
| `[ticket].valid_types` | Array<String> | `["read-only", "mutating", "destructive"]` | Các type hợp lệ |
| `[ticket].exclude_files` | Array<String> | `["TEMPLATE.md"]` | Files bỏ qua khi scan |
| `[[doc_structure]].file` | String | (none) | Path tới doc file cần check structure (relative docs_dir) |
| `[[doc_structure]].required_sections` | u32 | (none) | Số sections bắt buộc cho file đó |
| `[[doc_structure]].required_non_empty` | Array<u32> | `[]` | Sections phải có content |
| `[[count_check]].file` | String | (none) | Doc chứa số hardcoded |
| `[[count_check]].doc_pattern` | String | (none) | Regex (capture group 1) trích số từ doc |
| `[[count_check]].command` | String | (none) | Shell command để lấy số thực tế |
| `[[count_check]].command_pattern` | String | (none) | Regex (capture group 1) parse stdout |
| `[[count_check]].description` | String | `""` | Label trong error message |
| `[[cross_doc]].source` | String | (none) | Doc khai báo set giá trị chính thức |
| `[[cross_doc]].source_pattern` | String | (none) | Regex (capture group 1) trích values từ source |
| `[[cross_doc]].target` | String | (none) | Doc phải chứa tất cả values từ source |
| `[[cross_doc]].target_pattern` | String | (none) | Regex (capture group 1) trích values từ target |
| `[[cross_doc]].description` | String | `""` | Label trong error message |

Không có file config → dùng defaults. Mọi option đều optional.
`docs-gate init` scan project → auto-generate `.docs-gate.toml` phù hợp.
`[[doc_structure]]` là array — dùng nhiều entries để check nhiều file.

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
| Invalid regex (architecture) | Panic (bug, không phải user error) |
| Invalid regex (count/cross_doc user pattern) | CheckStatus::Fail("Invalid {kind}_pattern regex: {error}") |
| Section heading parse fail | Skip section, count as missing |
| Ticket dir không tồn tại | CheckStatus::Warn("Ticket directory not found: {path}") |
| Ticket file thiếu Type | CheckStatus::Fail("{filename}: missing Type declaration") |
| Ticket type không hợp lệ | CheckStatus::Fail("{filename}: invalid type '{value}'") |
| count_check command spawn fail | CheckStatus::Fail("Failed to run command: {error}") |
| count_check command non-zero exit | CheckStatus::Fail("Command failed with exit {code}: {stderr}") |
| count_check capture non-numeric | CheckStatus::Fail("{kind}_pattern captured non-numeric value: {value}") |
| count_check pattern miss | CheckStatus::Fail("{kind}_pattern did not match in {target}") |
| cross_doc source matches nothing | CheckStatus::Warn("source_pattern matched nothing in {source}") |
| cross_doc target missing values | CheckStatus::Fail("{label}: target missing values from source: {list}{ and N more}") |

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
- **Generic helper:** `check_doc_file(path, required_sections, required_non_empty, name_prefix)` — extract logic ra dùng chung. `check_architecture` và `check_doc_structure` đều gọi helper này.
- **doc_structure:** `check_doc_structure` iterate `config.doc_structure: Vec<DocStructureConfig>` — mỗi entry chạy helper với `name_prefix = "doc-{file}"` để tránh trùng tên với architecture results.
- **KHÔNG handle:** Nested subsections (### ) — chỉ count top-level ## sections

### discovery.rs
- **Algorithm:** Read file → find `## Discovery Report` heading → check 4 required `### ` sub-headings → check each has content
- **Content check:** Between heading and next heading, >= 1 non-empty, non-comment line
- **Comment detection:** Lines starting with `<!--` ignored
- **Complexity:** O(n) single pass
- **KHÔNG handle:** Discovery Report embedded in larger docs with conflicting headings

### ticket.rs
- **Algorithm:** Read ticket_dir → filter *.md, exclude exclude_files → for each: if type_pattern is Some, regex match → validate against valid_types. If type_pattern is None, skip type validation (pass all).
- **Regex:** Configurable via `ticket.type_pattern`. Default: `\*\*Type:\*\*\s*` followed by backtick-wrapped value
- **Complexity:** O(n*m) where n = number of files, m = avg file size
- **KHÔNG handle:** Multiple Type declarations with different values in same file — takes first valid match

### init.rs
- **Algorithm:** Scan CWD → detect docs dir, changelog, architecture, ticket dir → auto-detect type patterns + excludes → generate Config → write .docs-gate.toml
- **Type pattern detection:** Try known patterns (Type, Loại, Classification), then generic `**Key:** \`value\`` pattern. Pick pattern matching most files.
- **Architecture detection:** If file found, count `## N.` sections, set required_sections + required_non_empty. If not found, set enabled=false.
- **Exclude detection:** Files with "template", "readme", "example", "sample" in name (case-insensitive).
- **Output:** Summary to stderr, TOML content to stdout. Writes .docs-gate.toml to CWD.
- **KHÔNG handle:** Non-standard docs layouts. Nested ticket dirs. Multiple architecture files.

### config.rs
- **Algorithm:** Check `.docs-gate.toml` exists → parse → merge with defaults
- **Merge strategy:** Config values override defaults, missing values use defaults
- **Data structure:** Flat struct for existing keys + nested TicketConfig struct for `[ticket]` section
- **Backward compat:** Flat keys unchanged; `[ticket]` section optional with `#[serde(default)]`

### count.rs
- **Algorithm:** For each `[[count_check]]` entry: resolve doc path → read → regex extract first capture → parse u64 → spawn subprocess (`sh -c "{command}"` Unix, `cmd /C` Windows) → on success regex extract first capture from stdout → parse u64 → compare.
- **Path resolution:** Try absolute → `docs_dir/file` → repo root (CWD). First existing wins.
- **Error surface:** Every failure mode returns a Fail CheckResult with a precise reason. No panics for user-supplied regex.
- **Subprocess trust:** User opts in via config; same trust model as git hooks. No timeout, no whitelist. Document in README.
- **Complexity:** O(file size + command runtime) per entry.
- **KHÔNG handle:** Multi-capture comparison. Floating-point numbers. Command timeout. Streaming stdout (waits for completion).

### cross_doc.rs
- **Algorithm:** For each `[[cross_doc]]` entry: resolve source + target paths → read both → compile both regexes → collect HashSet<String> of capture group 1 from each → compute `source ∖ target` → if empty pass, else fail with sorted list capped at 5 with " and N more".
- **Direction:** Subset relationship. Target ⊇ Source. Target may have extra values.
- **Empty source:** Returns Warn (likely user regex error, not real drift). Empty target with non-empty source → Fail (genuine missing values).
- **Comparison:** Exact string match. Use `(?i)` in regex for case-insensitive needs.
- **Complexity:** O(n + m) where n, m = source and target file sizes.
- **KHÔNG handle:** Bidirectional checks. Fuzzy matching. Multi-capture-group comparison.

### mcp/server.rs + mcp/tools.rs
- **Architecture:** DocsGateServer struct holds `config_path: Option<PathBuf>` (NOT a parsed Config) + ToolRouter. #[tool_router] macro generates routing from tool name to handler method.
- **Tools:** 5 tools exposed via #[tool] macro. Each tool calls existing sync check functions directly — no logic duplication.
- **Hot-reload:** Each tool call invokes `self.load_fresh_config()` which calls `config::load_config(self.config_path.as_deref())` — re-reads `.docs-gate.toml` from disk every call. Editing the config no longer requires restarting the server.
- **Parameter resolution:** `resolve_config(Config, Option<String>)` takes the freshly-loaded config by value and applies the optional `docs_dir` override. No cloning needed — caller already owns a fresh Config.
- **Return format:** All tools return Json<Vec<CheckResult>> — rmcp serializes to JSON text content in MCP response.
- **Transport:** stdio only. stdout = JSON-RPC channel. All logs/errors → stderr.
- **Lifecycle:** Config path captured at startup; config content read fresh per call. Server runs until client disconnect.
- **KHÔNG handle:** HTTP/SSE transport. Resources or prompts (tools only).

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
- **Init mode (init):** Scan CWD → detect docs structure → generate .docs-gate.toml → exit. One-time setup.
- **Default mode:** Parse args → load config → run checks → exit. Nhanh, < 1 giây.
- **Watch mode (--watch):** Long-running process. Run checks → watch files → re-run on changes → Ctrl+C to exit.
  - Watcher: notify RecommendedWatcher (cross-platform) on docs_dir, + ticket_dir if --all
  - Debounce: 500ms hardcoded
  - Terminal: clear + timestamp before each re-run
  - Exit code: last check run's exit code
- **Serve mode (serve):** Long-running MCP server on stdio. JSON-RPC on stdin/stdout. Logs on stderr.
  - Config path captured at startup; `.docs-gate.toml` re-read on every tool call (hot-reload)
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

### Ticket type regex configurable
- **Modules liên quan:** checks/ticket.rs, config.rs
- **Vấn đề:** Type pattern giờ configurable qua `ticket.type_pattern`. Nhưng user vẫn phải cung cấp regex đúng.
- **Xử lý hiện tại:** `docs-gate init` auto-detect pattern. Nếu không detect được → set None (skip type check).

### Config file location cố định
- **Modules liên quan:** config.rs
- **Vấn đề:** Chỉ tìm .docs-gate.toml ở CWD. Không walk up directory tree.
- **Xử lý hiện tại:** Chấp nhận cho Phase 1. Thêm --config flag nếu cần.

### MCP server stdio only
- **Modules liên quan:** mcp/server.rs
- **Vấn đề:** Chỉ hỗ trợ stdio transport. Không SSE, không HTTP.
- **Xử lý hiện tại:** Đủ cho Claude Desktop/Code integration. Network transport ngoài scope.

### `[[doc_structure]]` reuses architecture parser
- **Modules liên quan:** checks/architecture.rs
- **Vấn đề:** Doc structure check dùng cùng regex `^## \d+\.` như architecture. Doc files dùng heading style khác (vd: `## A. Section`, không có số) sẽ không được parse.
- **Xử lý hiện tại:** Document rõ. Pattern này phù hợp với conventions của project Vietnamese (PROJECT.md, ARCHITECTURE.md numbered sections).

### `[[count_check]]` runs arbitrary shell commands
- **Modules liên quan:** checks/count.rs
- **Vấn đề:** Command từ config chạy qua `sh -c` (Unix) / `cmd /C` (Windows). User có quyền sửa config = quyền chạy bất kỳ command nào. Không sandbox, không whitelist.
- **Xử lý hiện tại:** Trust model giống git hooks: ai sửa được `.docs-gate.toml` đã có quyền code execution rồi. Document rõ trong README. KHÔNG sandbox vì sandboxing 1 dev tool đem lại an toàn giả.

### `[[count_check]]` không có timeout
- **Modules liên quan:** checks/count.rs
- **Vấn đề:** Command hang sẽ làm docs-gate treo. Không có timeout.
- **Xử lý hiện tại:** Chấp nhận. Dev tool, user Ctrl+C được. Có thể thêm config timeout ở phase sau nếu cần.
