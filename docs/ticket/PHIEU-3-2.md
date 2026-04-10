# Phiếu #3-2: MCP server skeleton + stdio transport

**Type:** `mutating`

**Khối lượng:** Vừa (1-2 ngày)

**Phụ thuộc:** Phiếu #3-1 phải merge trước.

---

## Tổng quan

Thêm `docs-gate serve` subcommand — khởi MCP server qua stdio transport. Server đăng ký tools nhưng chưa implement logic (shell). Phiếu này focus: protocol handshake hoạt động, tools listed, client kết nối được.

---

## Nhiệm vụ

### A. Dependencies (`Cargo.toml`)

Thêm:
```toml
rmcp = { version = "0.8", features = ["server", "transport-io", "macros"] }
schemars = "1.x"  # required by rmcp for tool parameter schemas
```

`schemars` là dependency bắt buộc của rmcp để generate JSON Schema cho tool parameters.

### B. MCP server module (`src/mcp/`)

**Tạo mới:** thư mục `src/mcp/` với 3 files:

```
src/mcp/
├── mod.rs       — pub mod server, tools; MCP entry point
├── server.rs    — DocsGateServer struct, ServerHandler impl
└── tools.rs     — Tool definitions (parameters + descriptions)
```

### C. Server struct (`src/mcp/server.rs`)

```rust
// DocsGateServer holds Config, implements rmcp::ServerHandler
pub struct DocsGateServer {
    config: Config,
}
```

**ServerHandler implementation:**
- `get_info()` → return ServerInfo với:
  - name: `"docs-gate"`
  - version: từ `env!("CARGO_PKG_VERSION")`
  - capabilities: `enable_tools()` only (không resources, không prompts)
- Tool handlers: 4 tools (xem section D)

### D. Tool definitions (`src/mcp/tools.rs`)

4 tools, mỗi tool dùng `#[tool]` macro từ rmcp:

| Tool name | Parameters | Description | Return |
|-----------|-----------|-------------|--------|
| `check_changelog` | `docs_dir: Option<String>` | Check CHANGELOG.md has recent entry | CheckResult as JSON text |
| `check_architecture` | `docs_dir: Option<String>` | Check ARCHITECTURE.md 9 sections + non-empty 7,8,9 | Vec<CheckResult> as JSON text |
| `check_discovery` | `file_path: String` (required) | Check Discovery Report format | Vec<CheckResult> as JSON text |
| `check_all` | `docs_dir: Option<String>` | Run all checks (changelog + architecture + tickets) | Vec<CheckResult> as JSON text |

**Parameter behavior:**
- `docs_dir` override Config.docs_dir nếu provided, dùng config default nếu None
- `file_path` cho discovery là required — trả error nếu thiếu

**Return format:** Mỗi tool trả về JSON text content chứa CheckResult(s) serialized. Thêm `#[derive(Serialize)]` cho CheckStatus và CheckResult nếu chưa có.

**Thợ verify:** CheckResult và CheckStatus hiện có `Serialize` derive không. Nếu chưa → thêm.

### E. CLI changes (`main.rs`)

Thêm subcommand `serve` vào clap enum:

```
docs-gate [--config path] [--verbose] [--watch] [--all]
docs-gate check-discovery <file>
docs-gate serve
```

**`serve` behavior:**
- Load config
- Tạo `DocsGateServer { config }`
- Start MCP server trên stdio transport: `server.serve(rmcp::transport::stdio()).await`
- KHÔNG output gì ra stdout (stdout là JSON-RPC channel)
- Log ra stderr nếu --verbose
- Chạy cho đến khi client disconnect hoặc process killed

**`serve` KHÔNG kết hợp với:**
- `--watch` → error, exit 2
- `--all` → không cần, MCP client gọi tool nào thì chạy tool đó

### F. Module map update

```
src/
├── main.rs
├── config.rs
├── watch.rs
├── mcp/
│   ├── mod.rs       — MCP module entry point
│   ├── server.rs    — DocsGateServer + ServerHandler impl
│   └── tools.rs     — Tool definitions with #[tool] macro
├── checks/
│   ├── mod.rs
│   ├── changelog.rs
│   ├── architecture.rs
│   ├── discovery.rs
│   └── ticket.rs
└── output.rs
```

---

## Files cần tạo/sửa

| File | Action | Gì |
|------|--------|----|
| `Cargo.toml` | Sửa | Thêm rmcp, schemars |
| `src/mcp/mod.rs` | Tạo mới | Module declarations |
| `src/mcp/server.rs` | Tạo mới | DocsGateServer struct + ServerHandler |
| `src/mcp/tools.rs` | Tạo mới | 4 tool definitions với #[tool] macro |
| `src/main.rs` | Sửa | Thêm `serve` subcommand routing |
| `src/checks/mod.rs` | Sửa | Thêm Serialize derive nếu thiếu |

---

## Dependencies mới

| Package | Version | Dùng cho |
|---------|---------|----------|
| rmcp | 0.8.x | MCP server SDK (features: server, transport-io, macros) |
| schemars | 1.x | JSON Schema generation cho tool parameters |

---

## Luật chơi

- Transport: CHỈ stdio. Không SSE, không HTTP. Single binary, no network.
- Tools gọi existing check functions trực tiếp — KHÔNG duplicate logic
- stdout khi serve = JSON-RPC only. Mọi log/debug → stderr
- Config load 1 lần lúc startup, KHÔNG hot-reload
- Tool parameters override config nhưng KHÔNG modify config object
- KHÔNG expose resources hay prompts — chỉ tools

---

## Kịch bản lỗi

- `serve` + `--watch` → stderr error, exit 2
- Client gọi tool không tồn tại → rmcp SDK tự handle (JSON-RPC error)
- Client gọi `check_discovery` thiếu `file_path` → tool return error text
- Config load fail → stderr warning, dùng defaults, server vẫn start
- Client disconnect → server exit clean

---

## Nghiệm thu

- [ ] `cargo build --release` zero warnings
- [ ] `docs-gate serve` — process starts, không crash, không output stdout
- [ ] `docs-gate serve --watch` → error, exit 2
- [ ] `docs-gate --help` — hiện `serve` subcommand
- [ ] MCP client test: connect qua stdio, `tools/list` → 4 tools listed với đúng name + description
- [ ] MCP client test: gọi `check_changelog` → nhận JSON response với CheckResult
- [ ] MCP client test: gọi `check_architecture` → nhận JSON response
- [ ] MCP client test: gọi `check_discovery` với file_path → nhận JSON response
- [ ] MCP client test: gọi `check_all` → nhận JSON response
- [ ] Existing CLI tests vẫn pass (regression)
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Tests: >= 4 unit tests (server info, tool parameter validation)
- [ ] Integration test: spawn process `docs-gate serve`, send JSON-RPC initialize, verify response

---

## Assumptions

- rmcp 0.8.x `#[tool]` macro tạo tool từ method trên struct — thợ verify syntax bằng rmcp docs/examples
- rmcp `ServerHandler` trait dùng `get_info()` để declare capabilities — thợ verify
- rmcp stdio transport: `rmcp::transport::stdio()` hoặc tương đương — thợ verify exact API
- CheckResult cần Serialize — thợ verify và thêm nếu thiếu
- tokio runtime đã có từ phiếu #3-1

---

## Docs cần update sau khi xong

- ARCHITECTURE.md Section 1: thêm mcp/ directory
- ARCHITECTURE.md Section 2: thêm DocsGateServer, tool signatures
- ARCHITECTURE.md Section 3: thêm MCP serve data flow
- ARCHITECTURE.md Section 5: thêm rmcp, schemars dependencies
- ARCHITECTURE.md Section 7: thêm implementation notes cho mcp/
- ARCHITECTURE.md Section 8: thêm serve mode runtime behavior
- ARCHITECTURE.md Section 9: thêm known constraints (stdio only, no hot-reload)
- CHANGELOG.md: ghi entry
- PROJECT.md: update MCP server mode status → ✅
