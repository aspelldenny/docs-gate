# Phiếu #5-3: `[[count_check]]` — drift số liệu hardcoded trong docs

**Type:** `mutating`

**Khối lượng:** Vừa (~2-3h). New module + new config + subprocess execution.

**Phụ thuộc:** Không.

---

## Tổng quan

**Mục tiêu:** Phát hiện khi doc claim 1 con số mà code/test thực tế cho số khác. Ví dụ điển hình:
- CLAUDE.md: "258/258 tests pass" — chạy `cargo test`, parse output, so với 258
- README.md: "78 cards in deck" — chạy `wc -l data/cards.json`, parse, so với 78
- ARCHITECTURE.md: "5 prompt builders" — chạy `grep -c 'export function build' src/lib/ai/prompts.ts`

**Tại sao:** Số liệu hardcoded là chỗ rot nhanh nhất. Code thay → quên update doc → doc nói dối.

**Approach:** Thêm `[[count_check]]` array. Mỗi entry:
1. Đọc doc, extract số bằng regex (capture group 1)
2. Chạy command (subprocess), parse stdout bằng regex
3. So sánh 2 số. Khác nhau → Fail.

---

## Nhiệm vụ

### A. Sửa `src/config.rs`

Thêm:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountCheckConfig {
    /// Doc file containing the hardcoded number (relative to docs_dir or repo root)
    pub file: String,
    /// Regex with one capture group to extract the number FROM the doc
    pub doc_pattern: String,
    /// Shell command to run for the actual count
    pub command: String,
    /// Regex with one capture group to extract the number from command stdout
    pub command_pattern: String,
    /// Human-readable label shown in error messages (e.g. "test count")
    #[serde(default)]
    pub description: String,
}
```

Field trong `Config`:

```rust
#[serde(default)]
pub count_check: Vec<CountCheckConfig>,
```

`Default::default()`: empty vec.

### B. New module `src/checks/count.rs`

```rust
pub fn check_counts(config: &Config) -> Vec<CheckResult> { ... }
```

Logic per entry:

1. Resolve `file` path: nếu absolute, dùng nguyên; nếu relative, thử `docs_dir/file` trước, nếu không tồn tại thì thử relative repo root (CWD).
2. Read file → regex `doc_pattern` → extract first capture group → parse as number. Fail nếu pattern không match hoặc không phải số.
3. Spawn subprocess: `sh -c "{command}"`. Capture stdout. Wait, có exit code.
4. Nếu command fail (non-zero exit) → Fail với stderr message.
5. Apply `command_pattern` lên stdout → extract → parse number.
6. So sánh: equal → Pass; khác → Fail với message rõ ("doc says X, command says Y").

CheckResult name: `count-{file}` hoặc `count-{description}` nếu có description.

### C. Wire vào `src/checks/mod.rs`

`run_all_checks(config)` gọi thêm `count::check_counts(config)` cuối cùng.

### D. Update `src/checks/mod.rs` exports + module declaration

Thêm `pub mod count;` vào checks module.

### E. README example

```toml
[[count_check]]
file = "CLAUDE.md"
doc_pattern = '(\d+)/\d+ tests pass'
command = "cargo test 2>&1 | grep 'test result'"
command_pattern = 'test result: ok\. (\d+) passed'
description = "test count"
```

---

## Files cần sửa/tạo

| File | Action | Gì |
|------|--------|----|
| `src/config.rs` | Sửa | Add `CountCheckConfig` + `count_check: Vec<...>` field + Default |
| `src/checks/count.rs` | **Tạo mới** | `check_counts(&Config) -> Vec<CheckResult>` |
| `src/checks/mod.rs` | Sửa | `pub mod count;` + call trong `run_all_checks` |
| `src/init.rs` | Sửa | Add `count_check: Vec::new()` vào `Config { ... }` init |
| `README.md` | Sửa | Add `[[count_check]]` example |

---

## Luật chơi

- **KHÔNG** thêm dependency mới. Dùng `std::process::Command` + `regex` đã có.
- **Subprocess execution = trust user.** Command chạy qua `sh -c`. User opt-in qua config — không khác gì git pre-commit hooks. Document security note rõ trong README.
- **Path resolution:** Doc file có thể nằm trong `docs_dir` hoặc repo root. Try cả hai, KHÔNG yêu cầu full path.
- **No network calls.** Command chạy local thôi (sh -c không restrict, nhưng không phải concern của tool này).
- **Timeout:** Hardcoded 30s. Command chạy quá → kill, Fail. (Dùng `wait_timeout` pattern như `tests/mcp_integration.rs::WaitTimeout`, hoặc đơn giản hơn: `Command::output()` blocks indefinitely — chấp nhận risk vì đây là dev tool, user chạy interactive.)

  → Quyết định: KHÔNG implement timeout ở phiếu này. Nếu command hang, user Ctrl+C. Đơn giản trước, optimize sau.

---

## Kịch bản lỗi

| Lỗi | Xử lý |
|------|--------|
| File không tồn tại | Fail "File not found: {path}" |
| `doc_pattern` invalid regex | Fail "Invalid doc_pattern regex: {error}" |
| `doc_pattern` không match trong file | Fail "doc_pattern did not match in {file}" |
| Capture group 1 không phải số | Fail "doc_pattern captured non-numeric: {value}" |
| Command spawn fail | Fail "Failed to run command: {error}" |
| Command exit non-zero | Fail "Command failed with exit {code}: {stderr first 200 chars}" |
| `command_pattern` invalid regex | Fail "Invalid command_pattern regex: {error}" |
| `command_pattern` không match | Fail "command_pattern did not match in command output" |
| Command capture không phải số | Fail "command_pattern captured non-numeric: {value}" |
| Numbers khác nhau | Fail "{description}: doc says {X}, command says {Y}" |
| Numbers bằng nhau | Pass |

---

## Nghiệm thu

- [ ] Doc nói "5 tests" + command output "5 passed" → Pass.
- [ ] Doc nói "5 tests" + command output "7 passed" → Fail rõ ràng số doc vs command.
- [ ] Doc file missing → Fail.
- [ ] Command non-zero exit → Fail với stderr.
- [ ] Pattern invalid → Fail rõ regex error.
- [ ] Empty `[[count_check]]` array → no-op.
- [ ] `cargo build --release` zero warnings.
- [ ] `cargo test` all pass + tests mới (≥5 cases).
- [ ] `cargo clippy -- -D warnings` clean.

---

## Assumptions (thợ verify bằng Discovery Report)

- `std::process::Command::new("sh").arg("-c").arg(cmd).output()` works trên macOS + Linux. Windows? — docs-gate hiện đã support Windows trong CI matrix (ARCHITECTURE.md section 5). Trên Windows `sh` không mặc định có. → Workaround: trên Windows dùng `cmd /C`. Hoặc đơn giản: chấp nhận limitation, document.
  - **Quyết định:** Dùng `sh -c` cho Unix, `cmd /C` cho Windows (`cfg(target_family = "unix")` vs `cfg(target_family = "windows")`).
- Regex patterns trong TOML strings không cần escape kỹ hơn TOML đã làm.
- `Config::default()` có `count_check: vec![]` không phá test_default_config (sẽ assertion fail nếu không update — fix khi build).
