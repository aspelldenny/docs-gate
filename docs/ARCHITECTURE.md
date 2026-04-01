# ARCHITECTURE.md — docs-gate

> 9 sections bắt buộc. Section 7, 8, 9 PHẢI có.
> Kiến trúc sư đọc file này để viết phiếu. Sai ở đây = phiếu sai.

---

## 1. Module Map

```
src/
├── main.rs          — Entry point: CLI arg parsing + orchestration
├── config.rs        — Load .docs-gate.toml, defaults, validation
├── checks/
│   ├── mod.rs       — CheckResult type + run_all_checks()
│   ├── changelog.rs — Check CHANGELOG.md has recent entry
│   └── architecture.rs — Check ARCHITECTURE.md 9 sections + non-empty 7,8,9
└── output.rs        — Format results: human-readable + exit code
```

---

## 2. Public API

```rust
// main.rs
fn main() -> ExitCode
  // Parse CLI args (clap) → load config → run checks → format output → exit

// config.rs
pub struct Config {
    pub docs_dir: PathBuf,          // default: "docs"
    pub changelog: String,          // default: "CHANGELOG.md"
    pub architecture: String,       // default: "ARCHITECTURE.md"
    pub required_sections: usize,   // default: 9
    pub required_non_empty: Vec<usize>, // default: [7, 8, 9]
    pub changelog_max_age_days: u32,    // default: 1
}
pub fn load_config(path: Option<&Path>) -> Config

// checks/mod.rs
pub enum CheckStatus { Pass, Fail(String), Warn(String) }
pub struct CheckResult { pub name: String, pub status: CheckStatus }
pub fn run_all_checks(config: &Config) -> Vec<CheckResult>

// checks/changelog.rs
pub fn check_changelog(config: &Config) -> CheckResult

// checks/architecture.rs
pub fn check_architecture(config: &Config) -> Vec<CheckResult>

// output.rs
pub fn format_results(results: &[CheckResult]) -> String
pub fn exit_code(results: &[CheckResult]) -> ExitCode
```

---

## 3. Data Flow

```
CLI args (clap)
  → load_config(.docs-gate.toml hoặc defaults)
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

Không có file config → dùng defaults. Mọi option đều optional.

---

## 5. Dependencies

| Package | Version | Dùng cho |
|---------|---------|----------|
| clap | 4.x | CLI argument parsing (derive feature) |
| serde | 1.x | Deserialize config |
| toml | 0.8.x | Parse .docs-gate.toml |
| regex | 1.x | Parse markdown headings |
| chrono | 0.4.x | Date parsing cho changelog age check |

---

## 6. Error Handling

| Lỗi | Xử lý |
|------|--------|
| File không tồn tại | CheckStatus::Fail("File not found: {path}") |
| File không đọc được | CheckStatus::Fail("Cannot read: {path}: {error}") |
| Config parse error | Stderr warning, dùng defaults |
| Invalid regex | Panic (bug, không phải user error) |
| Section heading parse fail | Skip section, count as missing |

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

### config.rs
- **Algorithm:** Check `.docs-gate.toml` exists → parse → merge with defaults
- **Merge strategy:** Config values override defaults, missing values use defaults
- **Data structure:** Plain struct, no HashMap

### output.rs
- **Format:** `✅ PASS: {name}` hoặc `❌ FAIL: {name} — {reason}`
- **Exit code:** 0 nếu zero Fail, 1 nếu >= 1 Fail. Warn không ảnh hưởng exit code.

---

## 8. Runtime Behavior ⛔ BẮT BUỘC

- **Process model:** Foreground, synchronous, single-threaded
- **Output:** stdout cho results, stderr cho warnings/errors
- **Exit codes:** 0 = all pass, 1 = any fail, 2 = config/usage error
- **Startup:** Parse args → load config → run → exit. Không daemon, không watch.
- **Signal handling:** Không cần (chạy nhanh, < 1 giây)
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

### Config file location cố định
- **Modules liên quan:** config.rs
- **Vấn đề:** Chỉ tìm .docs-gate.toml ở CWD. Không walk up directory tree.
- **Xử lý hiện tại:** Chấp nhận cho Phase 1. Thêm --config flag nếu cần.
