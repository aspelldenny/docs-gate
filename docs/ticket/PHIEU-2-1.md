# Phiếu #2-1: Discovery Report check + Ticket type classification check

**Type:** `mutating`

**Khối lượng:** Vừa (1-3 ngày). Gộp 2 features vì cùng pattern, cùng phase.

---

## Tổng quan

Thêm 2 checks mới vào docs-gate:
1. `check-discovery <file>` — subcommand riêng, check 1 file có đúng Discovery Report format
2. `--all` flag — bật ticket type classification check (scan `docs/ticket/*.md`)

---

## Nhiệm vụ

### A. Discovery Report check (`checks/discovery.rs`)

**Input:** 1 file markdown path (từ CLI subcommand)

**Check rules:**
- File phải chứa heading `## Discovery Report`
- File phải chứa 4 sub-headings (exact match):
  - `### Assumptions trong phiếu — ĐÚNG:`
  - `### Assumptions trong phiếu — SAI:`
  - `### Edge cases phát hiện thêm:`
  - `### Docs đã cập nhật:`
- Mỗi heading phải có content: ít nhất 1 non-empty line giữa heading này và heading tiếp theo
  - `- [...]` counts as content (ví dụ: `- Không có`)
  - Comment lines `<!-- -->` KHÔNG count
  - Whitespace-only lines KHÔNG count

**Signature:**
```rust
// checks/discovery.rs
pub fn check_discovery(file_path: &Path) -> Vec<CheckResult>
```

**Kịch bản lỗi:**
- File không tồn tại → Fail("File not found: {path}")
- Thiếu `## Discovery Report` heading → Fail("Missing '## Discovery Report' heading")
- Thiếu sub-heading → Fail("Missing section: '{heading}'")
- Sub-heading có nhưng no content → Fail("Section empty: '{heading}'")
- File hợp lệ → Pass

### B. Ticket type classification check (`checks/ticket.rs`)

**Input:** directory path (default `docs/ticket`, configurable)

**Check rules:**
- Scan tất cả `*.md` files trong `ticket_dir`
- Skip files trong `exclude_files` config (default: `["TEMPLATE.md"]`)
- Mỗi file PHẢI có ít nhất 1 type declaration
- Type declaration regex: `\*\*Type:\*\*\s*`(read-only|mutating|destructive)`
- Type value PHẢI nằm trong `valid_types` config (default: `["read-only", "mutating", "destructive"]`)

**Signature:**
```rust
// checks/ticket.rs
pub fn check_tickets(config: &Config) -> Vec<CheckResult>
```

**Kịch bản lỗi:**
- ticket_dir không tồn tại → Warn("Ticket directory not found: {path}") — KHÔNG fail
- ticket_dir trống (0 .md files) → Pass (nothing to check)
- File thiếu Type declaration → Fail("{filename}: missing Type declaration")
- Type value không hợp lệ → Fail("{filename}: invalid type '{value}', expected one of: {valid_types}")
- Tất cả files hợp lệ → Pass

### C. Config additions (`config.rs`)

Thêm vào `Config` struct:

```rust
// Thêm fields
pub ticket_dir: PathBuf,            // default: "docs/ticket"
pub valid_types: Vec<String>,       // default: ["read-only", "mutating", "destructive"]
pub exclude_files: Vec<String>,     // default: ["TEMPLATE.md"]
```

KHÔNG có `discovery_enabled` hay `ticket_enabled` — logic opt-in qua CLI:
- Discovery check: user gọi subcommand `check-discovery <file>` = opt-in
- Ticket check: user pass `--all` flag = opt-in

`.docs-gate.toml` format:
```toml
[ticket]
ticket_dir = "docs/ticket"
valid_types = ["read-only", "mutating", "destructive"]
exclude_files = ["TEMPLATE.md"]
```

Keys cũ (flat) giữ nguyên. Keys mới nằm dưới `[ticket]` section. Không phải breaking change.

Thợ verify: config.rs hiện dùng flat struct. Thêm nested struct cho `[ticket]` section với `#[serde(default)]`. Keys cũ không bị ảnh hưởng.

**Assumption (từ ARCHITECTURE.md Section 7):** Config dùng plain struct, merge with defaults. Thợ verify cách hiện tại handle partial config rồi quyết cách thêm nested section.

### D. CLI changes (`main.rs`)

Hiện tại: `docs-gate [--config path] [--verbose]`

Thêm:
- `docs-gate --all` — chạy existing checks + ticket check
- `docs-gate check-discovery <file>` — subcommand, check 1 file

**Assumption (từ ARCHITECTURE.md Section 2):** CLI dùng clap derive. Thợ verify cách thêm subcommand vào existing Args struct. clap hỗ trợ `#[command(subcommand)]` enum.

**Behavior:**
- `docs-gate` (không args) — chạy changelog + architecture checks (KHÔNG đổi behavior hiện tại)
- `docs-gate --all` — chạy changelog + architecture + ticket checks
- `docs-gate check-discovery <file>` — CHỈ chạy discovery check trên file đó, output + exit code riêng
- `--verbose` vẫn hoạt động với tất cả modes
- Discovery check KHÔNG chạy trong `--all` (vì cần file path cụ thể)

### E. Module wiring (`checks/mod.rs`)

- Thêm `pub mod discovery;` và `pub mod ticket;`
- `run_all_checks()` giữ nguyên behavior cũ (changelog + architecture)
- Thêm `run_all_checks_extended()` hoặc thêm param cho `run_all_checks()` — thợ tự quyết cách nào clean hơn, miễn behavior cũ KHÔNG thay đổi

---

## Files cần tạo/sửa

| File | Action | Gì |
|------|--------|----|
| `src/checks/discovery.rs` | Tạo mới | Discovery Report check logic |
| `src/checks/ticket.rs` | Tạo mới | Ticket type classification check logic |
| `src/checks/mod.rs` | Sửa | Thêm mod discovery, ticket + wiring |
| `src/config.rs` | Sửa | Thêm config fields cho discovery + ticket |
| `src/main.rs` | Sửa | Thêm --all flag + check-discovery subcommand |
| `src/output.rs` | Không đổi | Format hiện tại đã generic đủ |

---

## Luật chơi

- Pattern giống checks hiện có: read file → regex/string match → CheckResult
- KHÔNG thêm dependency mới — regex + std::fs đủ
- KHÔNG đổi behavior default `docs-gate` (backward compatible)
- Ticket dir không tồn tại = Warn, KHÔNG Fail (user có thể không dùng ticket system)
- Discovery check là subcommand riêng, KHÔNG chạy tự động

---

## Nghiệm thu

### Discovery check:
- [ ] File đúng format, 4 headings + content → Pass
- [ ] File thiếu 1 heading → Fail
- [ ] Heading có nhưng no content (chỉ whitespace) → Fail
- [ ] Heading có `- Không có` → Pass (counts as content)
- [ ] File không tồn tại → Fail
- [ ] Tests: >= 5 unit tests cho discovery.rs

### Ticket check:
- [ ] ticket_dir có 2 files, cả 2 có Type → Pass
- [ ] 1 file thiếu Type → Fail
- [ ] Type value không hợp lệ (ví dụ `Type: urgent`) → Fail
- [ ] ticket_dir không tồn tại → Warn (not Fail)
- [ ] ticket_dir trống → Pass
- [ ] TEMPLATE.md trong ticket_dir → bị skip, không fail
- [ ] Tests: >= 6 unit tests cho ticket.rs

### CLI:
- [ ] `docs-gate` — chạy như trước (changelog + architecture only)
- [ ] `docs-gate --all` — chạy thêm ticket check
- [ ] `docs-gate check-discovery somefile.md` — chạy discovery check
- [ ] `docs-gate --help` — hiện subcommand + flags mới
- [ ] Integration tests: >= 3 tests mới

### Config:
- [ ] `.docs-gate.toml` không có [ticket] section → dùng defaults
- [ ] `.docs-gate.toml` có `ticket_dir = "custom/path"` → dùng custom path
- [ ] `.docs-gate.toml` có `exclude_files = ["TEMPLATE.md", "DRAFT.md"]` → skip cả 2
- [ ] Backward compatible: config cũ không có fields mới → vẫn work

### Build:
- [ ] `cargo build --release` zero warnings
- [ ] `cargo test` all pass
- [ ] `cargo clippy -- -D warnings` clean

---

## Assumptions

- CLI hiện dùng clap derive — thợ verify tại `src/main.rs`
- Config hiện dùng flat struct — thợ verify tại `src/config.rs` cách handle nested TOML sections
- `run_all_checks()` return `Vec<CheckResult>` — thợ verify cách extend mà không break existing behavior
- output.rs format đã generic (`✅ PASS: {name}` / `❌ FAIL: {name} — {reason}`) — thợ verify không cần sửa

---

## Docs cần update sau khi xong

- ARCHITECTURE.md Section 1: thêm discovery.rs, ticket.rs
- ARCHITECTURE.md Section 2: thêm signatures mới
- ARCHITECTURE.md Section 3: update data flow cho --all và check-discovery
- ARCHITECTURE.md Section 4: thêm config options mới
- ARCHITECTURE.md Section 7: thêm implementation notes cho discovery.rs, ticket.rs
- CHANGELOG.md: ghi entry
