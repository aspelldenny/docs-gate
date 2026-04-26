# Phiếu #5-4: `[[cross_doc]]` — token consistency giữa 2 doc files

**Type:** `mutating`

**Khối lượng:** Vừa (~2-3h). New module + new config. KHÔNG subprocess.

**Phụ thuộc:** Không.

---

## Tổng quan

**Mục tiêu:** Verify token/identifier xuất hiện trong doc A cũng phải xuất hiện trong doc B. Ví dụ:
- PROJECT.md liệt kê các Phase → CLAUDE.md "Phase hiện tại" phải tham chiếu đúng các Phase đó.
- README.md API ref liệt kê endpoints → ARCHITECTURE.md Section 2 phải có cùng endpoints.
- BACKEND_GUIDE.md liệt kê models → CHANGELOG.md "Schema thay đổi" phải có entry.

**Tại sao:** Cross-doc inconsistency là loại drift im lặng — không có lỗi rõ ràng, chỉ docs nói chệch nhau. Manual review hay sót.

**Approach:** Thêm `[[cross_doc]]` array. Mỗi entry:
1. Source file → extract values bằng `source_pattern` (capture group 1, multi-match).
2. Target file → extract values bằng `target_pattern`.
3. Verify: target set ⊇ source set. Missing values → Fail.

**Quan trọng:** Quan hệ là **subset** (target chứa tất cả source values). Target có thể có thêm. Lý do: source thường là "danh sách chính thức" (vd: list features), target là "doc khác phải reflect". Target có thêm = OK (vd: target có notes).

---

## Nhiệm vụ

### A. Sửa `src/config.rs`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrossDocConfig {
    /// Source doc — declares the canonical set of values
    pub source: String,
    /// Regex with one capture group; all matches in source = canonical set
    pub source_pattern: String,
    /// Target doc — must contain all values from source
    pub target: String,
    /// Regex with one capture group; all matches in target = "what target has"
    pub target_pattern: String,
    /// Human-readable label shown in error messages
    #[serde(default)]
    pub description: String,
}
```

Field trong `Config`:

```rust
#[serde(default)]
pub cross_doc: Vec<CrossDocConfig>,
```

`Default::default()`: empty vec.

### B. New module `src/checks/cross_doc.rs`

```rust
pub fn check_cross_doc(config: &Config) -> Vec<CheckResult> { ... }
```

Logic per entry:

1. Resolve `source` path: try `docs_dir/source` → if missing, try repo root.
2. Resolve `target` path: same logic.
3. Read both files.
4. Compile both regexes (Fail nếu invalid).
5. Extract source set: collect all `captures_iter` capture group 1 → HashSet<String>.
6. Extract target set: same.
7. Compute missing: source ∖ target (values in source not in target).
8. If missing empty → Pass. Else → Fail với danh sách missing values.

CheckResult name: `cross-{source}-{target}` hoặc `cross-{description}` nếu có.

### C. Wire vào `src/checks/mod.rs`

`run_all_checks(config)` gọi thêm `cross_doc::check_cross_doc(config)`.

### D. Update `src/checks/mod.rs` exports

Thêm `pub mod cross_doc;`.

### E. README example

```toml
[[cross_doc]]
source = "PROJECT.md"
source_pattern = '\| Phase ([0-9]+) '
target = "CLAUDE.md"
target_pattern = 'Phase ([0-9]+) '
description = "Phase numbers consistency"
```

---

## Files cần sửa/tạo

| File | Action | Gì |
|------|--------|----|
| `src/config.rs` | Sửa | Add `CrossDocConfig` + `cross_doc: Vec<...>` field + Default |
| `src/checks/cross_doc.rs` | **Tạo mới** | `check_cross_doc(&Config) -> Vec<CheckResult>` |
| `src/checks/mod.rs` | Sửa | `pub mod cross_doc;` + call trong `run_all_checks` |
| `src/init.rs` | Sửa | Add `cross_doc: Vec::new()` vào `Config { ... }` init |
| `README.md` | Sửa | Add `[[cross_doc]]` example |

---

## Luật chơi

- **KHÔNG** thêm dependency mới. Dùng `regex` đã có.
- **KHÔNG subprocess.** Pure file read + regex.
- **Path resolution:** Same logic như P5-3 — try `docs_dir/file` trước, fallback repo root.
- **Direction:** target ⊇ source (subset, target có thể có thêm). Hard rule, không config bidirectional.
- **Empty source matches:** Nếu `source_pattern` không match gì → Warn (regex có thể sai). Không Fail.
- **Empty target matches:** Nếu `target_pattern` không match gì NHƯNG source có matches → Fail (target trống thì rõ ràng missing).
- **Comparison:** String exact match, case-sensitive. User responsibility nếu cần case-insensitive (dùng `(?i)` trong regex).

---

## Kịch bản lỗi

| Lỗi | Xử lý |
|------|--------|
| `source` file không tồn tại | Fail "Source file not found: {path}" |
| `target` file không tồn tại | Fail "Target file not found: {path}" |
| `source_pattern` invalid regex | Fail "Invalid source_pattern regex: {error}" |
| `target_pattern` invalid regex | Fail "Invalid target_pattern regex: {error}" |
| Source pattern 0 match | Warn "source_pattern matched nothing in {source}" |
| Target chứa hết source | Pass |
| Target thiếu N values | Fail "{description}: target missing values from source: {value1, value2, ...}" (cap ở 5 values + "and N more" nếu nhiều) |

---

## Nghiệm thu

- [ ] Source có {A, B, C}, target có {A, B, C, D} → Pass (target có thêm OK).
- [ ] Source có {A, B, C}, target có {A, B} → Fail liệt kê C missing.
- [ ] Source pattern không match → Warn rõ ràng.
- [ ] Target file missing → Fail rõ ràng.
- [ ] Empty `[[cross_doc]]` → no-op.
- [ ] Multiple entries → mỗi entry result độc lập.
- [ ] `cargo build --release` zero warnings.
- [ ] `cargo test` all pass + tests mới (≥5 cases).
- [ ] `cargo clippy -- -D warnings` clean.

---

## Assumptions (thợ verify bằng Discovery Report)

- `regex::Regex::captures_iter` trả về tất cả non-overlapping matches.
- HashSet<String> hashing đủ nhanh cho các doc cỡ 100KB (~ms).
- TOML deserialize `[[cross_doc]]` thành `Vec<CrossDocConfig>` — same pattern như `[[doc_structure]]` đã làm work.
- `Config::default()` thêm `cross_doc: vec![]` không phá test_default_config (compiler sẽ bắt nếu sót).
