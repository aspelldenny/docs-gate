# Phiếu #5-2: Generic `[[doc_structure]]` array — check N file với required sections

**Type:** `mutating`

**Khối lượng:** Vừa (1-2h). Refactor existing architecture check + add new config array.

**Phụ thuộc:** Không có (độc lập với P5-1, có thể làm song song hoặc tuần tự).

---

## Tổng quan

**Mục tiêu:** Hiện tại docs-gate chỉ check 1 file (`ARCHITECTURE.md`) cho required sections. Mở rộng để check **N file** (vd: TEST_CASES.md, AUDIT_PROTOCOL.md, ...) bằng config array generic.

**Tại sao:** Dự án thực tế (vd: tarot) có nhiều doc file cần đảm bảo cấu trúc — không chỉ ARCHITECTURE. Hiện không có cách dùng docs-gate enforce điều này.

**Approach:** Thêm `[[doc_structure]]` array vào config. Mỗi entry là 1 file cần check. Logic check **dùng lại** từ `architecture::check_architecture` (refactor thành helper generic).

**Backward compat:** `[architecture]` cũ vẫn hoạt động 100%. Internally, có thể coi nó như 1 entry đặc biệt, nhưng không phá schema cũ.

---

## Nhiệm vụ

### A. Sửa `src/config.rs`

Thêm:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocStructureConfig {
    /// Path to doc file relative to docs_dir (e.g. "TEST_CASES.md")
    pub file: String,
    /// Number of `## ` headings required
    pub required_sections: usize,
    /// 1-indexed section numbers required to be non-empty
    #[serde(default)]
    pub required_non_empty: Vec<usize>,
}
```

Thêm field vào `Config`:

```rust
#[serde(default)]
pub doc_structure: Vec<DocStructureConfig>,
```

`Default` impl: empty vec.

### B. Refactor `src/checks/architecture.rs`

Tách logic ra hàm helper:

```rust
pub fn check_doc_file(
    docs_dir: &Path,
    file: &str,
    required_sections: usize,
    required_non_empty: &[usize],
    name_prefix: &str,
) -> Vec<CheckResult> { ... }
```

`check_architecture(&Config)` gọi `check_doc_file` với name_prefix="architecture".

Thêm hàm mới:

```rust
pub fn check_doc_structure(config: &Config) -> Vec<CheckResult> {
    let mut results = Vec::new();
    for entry in &config.doc_structure {
        let prefix = format!("doc-{}", entry.file);
        results.extend(check_doc_file(
            &config.docs_dir,
            &entry.file,
            entry.required_sections,
            &entry.required_non_empty,
            &prefix,
        ));
    }
    results
}
```

### C. Sửa `src/checks/mod.rs`

`run_all_checks(config)` gọi thêm `architecture::check_doc_structure(config)` sau architecture check:

```rust
results.extend(architecture::check_architecture(config));
results.extend(architecture::check_doc_structure(config));
```

### D. Sửa MCP server

Tool `check_architecture` hiện chỉ check architecture file. Quyết định: **giữ nguyên tool `check_architecture`** (chỉ check ARCHITECTURE.md để backward compat). `check_all` tự động pick up doc_structure mới.

Nếu cần MCP tool riêng cho doc_structure, làm phiếu sau. Phiếu này KHÔNG thêm tool mới.

### E. Update README.md

Thêm example config trong section Configuration:

```toml
[[doc_structure]]
file = "TEST_CASES.md"
required_sections = 3
required_non_empty = [1, 2, 3]

[[doc_structure]]
file = "AUDIT_PROTOCOL.md"
required_sections = 6
required_non_empty = [1, 2, 3, 4, 5, 6]
```

---

## Files cần sửa

| File | Action | Gì |
|------|--------|----|
| `src/config.rs` | Sửa | Add `DocStructureConfig` struct + `doc_structure: Vec<...>` field |
| `src/checks/architecture.rs` | Sửa | Extract helper `check_doc_file`, add `check_doc_structure` |
| `src/checks/mod.rs` | Sửa | `run_all_checks` gọi thêm `check_doc_structure` |
| `README.md` | Sửa | Add example `[[doc_structure]]` config |

---

## Luật chơi

- KHÔNG đổi `[architecture]` schema cũ. `enabled`, `file`, `required_sections`, `required_non_empty` giữ nguyên.
- KHÔNG đổi MCP tool interface. `check_architecture` tool vẫn chỉ check ARCHITECTURE.md.
- KHÔNG đổi CheckResult format.
- `[[doc_structure]]` là OPTIONAL — config không có thì không có entry → no-op.
- Tên `name` field của CheckResult cho doc_structure: `doc-{file}` + suffix `-section-{n}` cho từng section. Tránh trùng với architecture (`architecture` + `architecture-section-{n}`).

---

## Kịch bản lỗi

- File trong `[[doc_structure]]` không tồn tại → CheckResult Fail (giống architecture check khi file missing).
- File rỗng → Fail.
- Section count thiếu → Fail.
- Required section trống → Fail.
- `required_non_empty = []` (không required) → chỉ check section count.
- `required_sections = 0` → trivially pass (không check gì).

---

## Nghiệm thu

- [ ] Config có `[[doc_structure]]` với 2 entries → cả 2 file được check, kết quả riêng biệt.
- [ ] File trong doc_structure missing → Fail rõ ràng (tên file trong message).
- [ ] Config không có `[[doc_structure]]` → run_all_checks chạy như cũ, không thêm result.
- [ ] `[architecture]` vẫn check ARCHITECTURE.md độc lập với doc_structure.
- [ ] `cargo build --release` zero warnings.
- [ ] `cargo test` all pass + tests mới cho doc_structure (single, multiple, missing file, empty config).
- [ ] `cargo clippy -- -D warnings` clean.
- [ ] README có example `[[doc_structure]]`.

---

## Assumptions (thợ verify bằng Discovery Report)

- `architecture::check_architecture` hiện tại có thể tách helper không phá tests cũ.
- `Config::default()` thêm `doc_structure: vec![]` không phá `test_default_config`.
- TOML deserialization của `Vec<DocStructureConfig>` từ `[[doc_structure]]` array works out of the box với serde.
- Không có MCP tool nào hiện tại expect 1 specific result count từ architecture (verify trong tests/mcp_integration.rs).
