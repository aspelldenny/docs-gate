# Phiếu #3-3: MCP end-to-end testing + documentation

**Type:** `read-only`

**Khối lượng:** Nhỏ (< 1 ngày)

**Phụ thuộc:** Phiếu #3-2 phải merge trước.

---

## Tổng quan

End-to-end testing cho MCP server mode. Viết comprehensive integration tests, tạo Claude Desktop config example, update README với MCP usage instructions. KHÔNG thêm code mới — chỉ tests + docs.

---

## Nhiệm vụ

### A. Integration tests (`tests/mcp_integration.rs`)

**Tạo mới:** `tests/mcp_integration.rs`

Test strategy: spawn `docs-gate serve` process, giao tiếp qua stdin/stdout JSON-RPC.

**Tests cần viết:**

1. **Initialize handshake:** Send `initialize` request → verify response có `serverInfo.name == "docs-gate"`, `capabilities.tools` present
2. **Tools list:** Send `tools/list` → verify 4 tools: `check_changelog`, `check_architecture`, `check_discovery`, `check_all`
3. **Tool call — check_changelog pass:** Setup temp dir có CHANGELOG.md hợp lệ → call `check_changelog` với `docs_dir` → verify Pass result
4. **Tool call — check_changelog fail:** Setup temp dir không có CHANGELOG.md → call `check_changelog` → verify Fail result
5. **Tool call — check_architecture pass:** Setup temp dir có ARCHITECTURE.md 9 sections → call `check_architecture` → verify Pass
6. **Tool call — check_discovery pass:** Setup temp dir có discovery file hợp lệ → call `check_discovery` với `file_path` → verify Pass
7. **Tool call — check_discovery missing path:** Call `check_discovery` không có `file_path` → verify error response
8. **Tool call — check_all:** Setup temp dir đầy đủ → call `check_all` → verify combined results
9. **Graceful shutdown:** Send shutdown hoặc close stdin → verify process exits clean

**Helper cần viết:**
- `fn spawn_server() -> Child` — spawn docs-gate serve process
- `fn send_jsonrpc(stdin, request) -> Response` — send request, read response
- `fn setup_valid_docs(temp_dir)` — tạo CHANGELOG.md + ARCHITECTURE.md hợp lệ
- `fn setup_valid_discovery(temp_dir)` — tạo discovery file hợp lệ

### B. README.md update

Thêm sections vào README.md:

1. **MCP Server Mode** — cách chạy `docs-gate serve`
2. **Claude Desktop Configuration** — example `claude_desktop_config.json`:
   ```json
   {
     "mcpServers": {
       "docs-gate": {
         "command": "docs-gate",
         "args": ["serve"],
         "cwd": "/path/to/your/project"
       }
     }
   }
   ```
3. **Watch Mode** — cách dùng `docs-gate --watch`
4. **Available MCP Tools** — bảng 4 tools + parameters + description

### C. Example config (`examples/claude_desktop_config.json`)

Tạo file example để user copy-paste.

---

## Files cần tạo/sửa

| File | Action | Gì |
|------|--------|----|
| `tests/mcp_integration.rs` | Tạo mới | 9+ integration tests cho MCP server |
| `README.md` | Sửa | Thêm MCP + watch mode docs |
| `examples/claude_desktop_config.json` | Tạo mới | Example Claude Desktop config |

---

## Luật chơi

- KHÔNG thêm code mới vào src/ — chỉ tests + docs
- Integration tests dùng tempfile crate (đã có trong dev-dependencies)
- JSON-RPC messages phải đúng MCP spec: `jsonrpc: "2.0"`, `id`, `method`, `params`
- Tests phải timeout sau 5 giây — tránh hang nếu server không respond
- README viết tiếng Anh (open source target)

---

## Kịch bản lỗi

- Server process không start → test fail với message rõ ràng
- Server không respond trong 5s → test timeout, fail
- JSON-RPC parse error → test verify error response format

---

## Nghiệm thu

- [ ] `cargo test` — tất cả tests pass, kể cả MCP integration tests
- [ ] Integration tests cover: initialize, tools/list, 4 tool calls (pass + fail), shutdown
- [ ] README.md có instructions cho MCP server mode + Claude Desktop config
- [ ] `examples/claude_desktop_config.json` tồn tại và valid JSON
- [ ] `cargo clippy -- -D warnings` clean
- [ ] Tests: >= 9 integration tests mới
- [ ] Tổng test count: >= 58 tests (49 hiện có + 9 mới)

---

## Assumptions

- MCP JSON-RPC initialize request format theo spec 2025-11-25 — thợ verify tại modelcontextprotocol.io
- rmcp server tự handle JSON-RPC framing trên stdio — thợ verify không cần manual framing
- tempfile crate đã có trong dev-dependencies từ Phase 1 — thợ verify
- Binary name `docs-gate` available sau `cargo build` — thợ verify binary name trong Cargo.toml

---

## Docs cần update sau khi xong

- CHANGELOG.md: ghi entry (tests + docs)
- PROJECT.md: update Phase 3 status → ✅
