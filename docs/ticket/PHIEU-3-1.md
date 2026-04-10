# Phiếu #3-1: Async migration + Watch mode

**Type:** `mutating`

**Khối lượng:** Vừa (1-2 ngày)

---

## Tổng quan

Migrate main.rs sang tokio async runtime (chuẩn bị cho MCP ở phiếu #3-2). Thêm `--watch` flag: re-run checks khi file thay đổi. CLI one-shot behavior KHÔNG đổi.

---

## Nhiệm vụ

### A. Async migration (`main.rs`, `Cargo.toml`)

- Thêm dependency: `tokio` (features: `["rt", "macros"]`) — KHÔNG dùng `"full"`, chỉ features cần thiết
- Thêm dependency: `notify` 8.x (file watcher cho watch mode)
- `main()` → `#[tokio::main] async fn main()` 
- Tất cả check functions giữ nguyên sync — KHÔNG refactor checks thành async
- Chỉ main entry point là async, checks chạy sync bên trong như cũ
- `ExitCode` return giữ nguyên behavior: 0 all pass, 1 any fail, 2 config error

**Thợ verify:** main.rs hiện dùng `fn main() -> ExitCode`. Confirm clap derive vẫn work với `#[tokio::main]`.

### B. Watch mode (`src/watch.rs`)

**Tạo mới:** `src/watch.rs`

**Behavior:**
- `docs-gate --watch` — chạy checks lần đầu, rồi watch file changes, re-run khi detect thay đổi
- `docs-gate --watch --all` — watch mode + ticket checks
- Watch targets: `{docs_dir}/CHANGELOG.md`, `{docs_dir}/ARCHITECTURE.md`, và nếu `--all` thì cả `{ticket_dir}/*.md`
- Debounce: 500ms — tránh re-run nhiều lần khi editor save (notify crate fire nhiều events)
- Output: clear terminal (`\x1B[2J\x1B[H`) trước mỗi re-run, hiện timestamp + results
- Ctrl+C để thoát — tokio signal handling
- `--watch` KHÔNG hoạt động với `check-discovery` subcommand (subcommand check 1 file cụ thể, watch không có nghĩa)

**Signature:**
```rust
// watch.rs
pub async fn run_watch(config: &Config, extended: bool) -> ExitCode
```

**Algorithm:**
1. Run checks lần đầu, print results
2. Setup notify watcher trên docs_dir (recursive) + ticket_dir nếu extended
3. Loop: recv event → debounce 500ms → re-run checks → print results
4. Ctrl+C → clean exit

### C. CLI changes (`main.rs`)

Thêm flag `--watch` vào Args struct (clap derive):
```
docs-gate [--config path] [--verbose] [--watch] [--all]
docs-gate check-discovery <file>
```

**Logic:**
- `--watch` + default mode → `run_watch(config, false)`
- `--watch --all` → `run_watch(config, true)`
- `--watch` + `check-discovery` → error: "Watch mode not supported with check-discovery subcommand"
- Không có `--watch` → behavior hiện tại, không thay đổi gì

### D. Module map update

```
src/
├── main.rs          — Entry point: CLI arg parsing + orchestration (async)
├── config.rs        — Load .docs-gate.toml, defaults, validation
├── watch.rs         — Watch mode: file watcher + re-run loop
├── checks/
│   ├── mod.rs       — CheckResult type + run_all_checks() + run_all_checks_extended()
│   ├── changelog.rs
│   ├── architecture.rs
│   ├── discovery.rs
│   └── ticket.rs
└── output.rs        — Format results: human-readable + exit code
```

---

## Files cần tạo/sửa

| File | Action | Gì |
|------|--------|----|
| `Cargo.toml` | Sửa | Thêm tokio, notify dependencies |
| `src/main.rs` | Sửa | `#[tokio::main]`, thêm --watch flag, route to watch mode |
| `src/watch.rs` | Tạo mới | Watch loop: notify watcher + debounce + re-run |

---

## Dependencies mới

| Package | Version | Dùng cho |
|---------|---------|----------|
| tokio | 1.x (features: `rt`, `macros`, `signal`) | Async runtime + Ctrl+C handling |
| notify | 8.x | Filesystem watcher cho watch mode |

---

## Luật chơi

- KHÔNG refactor checks thành async — giữ sync, chạy trong tokio::spawn_blocking nếu cần
- KHÔNG đổi behavior CLI hiện tại khi không có --watch
- Debounce 500ms cứng, KHÔNG configurable (Phase 3 scope)
- Watch mode exit code: exit code của lần check CUỐI CÙNG trước Ctrl+C
- stdout output watch mode: `[HH:MM:SS] Running checks...` + results
- notify watcher dùng `RecommendedWatcher` (cross-platform)

---

## Kịch bản lỗi

- `--watch` + `check-discovery` → stderr error message, exit 2
- notify watcher fail (permission denied) → stderr error, exit 2
- File bị xóa trong khi watch → re-run checks sẽ report Fail("File not found"), watch tiếp tục
- docs_dir không tồn tại → Fail checks lần đầu, watch vẫn chạy (user có thể tạo dir sau)

---

## Nghiệm thu

- [ ] `cargo build --release` zero warnings
- [ ] `docs-gate` (không --watch) — behavior KHÔNG đổi, exit code đúng
- [ ] `docs-gate --watch` — chạy checks, rồi watch, re-run khi sửa CHANGELOG.md
- [ ] `docs-gate --watch --all` — watch cả ticket_dir
- [ ] `docs-gate --watch check-discovery file.md` → error, exit 2
- [ ] Ctrl+C thoát clean (không panic, không orphan process)
- [ ] Debounce hoạt động: save file 3 lần nhanh → chỉ re-run 1 lần
- [ ] `cargo test` all existing tests vẫn pass (regression)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Tests: >= 3 unit tests cho watch.rs (debounce logic, config routing)
- [ ] Integration test: binary chạy --watch, send SIGINT, verify clean exit

---

## Assumptions

- main.rs hiện dùng `fn main() -> ExitCode` với clap derive — thợ verify async migration không break clap
- checks/mod.rs `run_all_checks()` và `run_all_checks_extended()` là sync — thợ verify không cần đổi
- output.rs `format_results()` là sync — thợ verify không cần đổi
- Config struct không cần thêm field cho watch (debounce hardcoded)

---

## Docs cần update sau khi xong

- ARCHITECTURE.md Section 1: thêm watch.rs
- ARCHITECTURE.md Section 2: thêm `run_watch()` signature
- ARCHITECTURE.md Section 3: thêm watch mode data flow
- ARCHITECTURE.md Section 5: thêm tokio, notify dependencies
- ARCHITECTURE.md Section 7: thêm implementation notes cho watch.rs
- ARCHITECTURE.md Section 8: thêm watch mode runtime behavior (process model, signal handling)
- CHANGELOG.md: ghi entry
- PROJECT.md: update watch mode status → ✅
