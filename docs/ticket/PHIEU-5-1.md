# Phiếu #5-1: MCP server config hot-reload

**Type:** `mutating`

**Khối lượng:** Nhỏ (< 1h). Bug fix focused, không thêm feature.

---

## Tổng quan

**Bug:** MCP server cache `.docs-gate.toml` lúc startup, không reload khi Sếp sửa config. Mỗi lần đổi config phải restart MCP server — UX kém + gây nhầm lẫn (CLI thấy config mới, MCP thấy config cũ).

**Root cause:** `src/mcp/server.rs:26` — `DocsGateServer::new(config)` lưu `Config` (đã parse) vào struct. `src/mcp/tools.rs:17` — `resolve_config(base, ...)` clone từ `base` cached → không bao giờ re-read file.

**Fix:** Reload `.docs-gate.toml` mỗi tool call thay vì cache 1 lần. Perf hit milliseconds, đổi lại luôn fresh.

---

## Nhiệm vụ

### A. Sửa `src/mcp/server.rs`

Thay vì lưu `Config`, lưu `Option<PathBuf>` (path tới `.docs-gate.toml`, lấy từ `--config` flag nếu có).

```rust
pub struct DocsGateServer {
    config_path: Option<PathBuf>,
    tool_router: ToolRouter<Self>,
}

impl DocsGateServer {
    pub fn new(config_path: Option<PathBuf>) -> Self {
        let tool_router = Self::tool_router();
        Self { config_path, tool_router }
    }
}
```

Mỗi tool method gọi `crate::config::load_config(self.config_path.as_deref())` để lấy fresh config trước khi gọi `resolve_config`.

### B. Sửa `src/mcp/tools.rs`

`resolve_config` đổi signature: nhận `Config` by value (đã load fresh) thay vì `&Config`. Hoặc giữ `&Config` nhưng caller load fresh trước khi gọi.

Đơn giản nhất: giữ logic cũ, chỉ đổi ở server.rs (load fresh → pass vào resolve_config).

### C. Sửa `src/main.rs`

`Some(Commands::Serve)` branch: pass `cli.config` (Option<PathBuf>) vào `DocsGateServer::new` thay vì `config` (Config đã parse).

---

## Files cần sửa

| File | Action | Gì |
|------|--------|----|
| `src/mcp/server.rs` | Sửa | Lưu `config_path: Option<PathBuf>` thay vì `config: Config`. Mỗi tool method load fresh. |
| `src/main.rs` | Sửa | Serve branch pass `cli.config` vào `DocsGateServer::new`. |
| `src/mcp/tools.rs` | (Có thể không sửa) | Giữ resolve_config signature cũ nếu được. |

---

## Luật chơi

- KHÔNG đổi MCP tool interface (tool names, params, return format).
- KHÔNG đổi `Config` struct (chỉ đổi cách MCP server hold reference).
- Backward compat 100% với CLI mode — CLI vẫn load config 1 lần như cũ.
- Perf hit chấp nhận được (mili-giây) — file IO không phải bottleneck cho dev tool.

---

## Kịch bản lỗi

- Nếu `.docs-gate.toml` không tồn tại → fallback `Config::default()` (giống logic load_config hiện tại).
- Nếu `.docs-gate.toml` có syntax error → fallback `Config::default()` + eprintln warning (giống hiện tại).
- Tool call vẫn return result hợp lệ kể cả config invalid.

---

## Nghiệm thu

- [ ] Sửa `.docs-gate.toml` trong khi MCP server đang chạy → call `check_all` lần kế → kết quả phản ánh config mới (KHÔNG cần restart).
- [ ] CLI mode (`docs-gate --all`) hoạt động như cũ.
- [ ] `cargo build --release` zero warnings.
- [ ] `cargo test` all pass (bao gồm test mới cho reload behavior).
- [ ] `cargo clippy -- -D warnings` clean.
- [ ] Test mới: spawn server với config A, modify config thành B, call tool, verify result reflect B.

---

## Assumptions (thợ verify bằng Discovery Report)

- `config::load_config(Option<&Path>)` đã có và xử lý đầy đủ edge cases (file missing, invalid TOML).
- `DocsGateServer` chỉ được dùng từ `Commands::Serve` branch trong main.rs.
- Không có code khác phụ thuộc vào field `config: Config` của `DocsGateServer` (đang `#[allow(dead_code)]`).
