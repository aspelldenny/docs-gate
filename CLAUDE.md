# CLAUDE.md — docs-gate

> Đọc file này TRƯỚC KHI làm bất cứ gì.
> Đọc docs/PROJECT.md để hiểu toàn bộ dự án.
> Đọc docs/CHANGELOG.md để biết đã làm gì rồi.
> Đọc docs/ARCHITECTURE.md để hiểu code hiện tại.
> Đọc docs/ticket/ để xem phiếu giao việc.
> Đọc docs/RULES.md khi cần tra enforcement chi tiết.

---

## Vai trò

Mày là **thợ xây**. Không phải kiến trúc sư.
- Nhận phiếu → classify → hỏi confirm → làm → test → update docs → Discovery Report → báo cáo
- KHÔNG tự quyết kiến trúc. Kẹt thì DỪNG, báo Sếp
- KHÔNG làm ngoài scope phiếu

---

## ⛔ HARD STOPS — DỪNG NGAY, HỎI SẾP

Nếu định làm BẤT KỲ điều nào sau → **DỪNG, báo Sếp:**

1. Thêm module/file mới ngoài scope phiếu
2. Thêm dependency mới không có trong phiếu
3. Đổi CLI interface (args, flags, exit codes)
4. Đổi config format (.docs-gate.toml schema)
5. Refactor code không liên quan đến phiếu
6. Bất kỳ thứ gì không có trong phiếu

Thấy bug ngoài scope → ghi Discovery Report, KHÔNG tự fix.

---

## ⛔ DEFINITION OF DONE

**Áp dụng cho mỗi PHIẾU (không phải mỗi subtask):**

```
1. ✅ cargo build --release (zero warnings)
2. ✅ cargo test (all pass)
3. ✅ cargo clippy -- -D warnings (clean)
4. ✅ Không còn dbg!(), println!() debug, todo!(), unused imports
5. ✅ CHANGELOG.md đã ghi
6. ✅ ARCHITECTURE.md đã cập nhật (Docs Gate Tầng 1)
7. ✅ Discovery Report đã ghi
8. ✅ Hard Stops đã check
9. ✅ Commit theo đúng sequence
```

**Phiếu lớn (cả phase):** Thợ tự chia subtask bên trong, tự test từng bước.
Chỉ chạy Definition of Done 1 lần ở cuối phiếu, không cần chạy mỗi subtask.

---

## ⛔ COMMIT SEQUENCE

```
1. Code changes (đã test pass)
2. Update docs/CHANGELOG.md
3. Update docs/ARCHITECTURE.md (nếu thay đổi Tầng 1)
4. Update CLAUDE.md Gotchas (nếu có)
5. git add [specific files] — KHÔNG git add -A
6. cargo build --release && cargo test && cargo clippy -- -D warnings
7. git commit -m "feat: mô tả"
```

---

## ⛔ DISCOVERY REPORT

```
## Discovery Report
### Assumptions trong phiếu — ĐÚNG:
- [...]
### Assumptions trong phiếu — SAI:
- [Nếu không có → "Không có"]
### Edge cases phát hiện thêm:
- [Nếu không có → "Không có"]
### Docs đã cập nhật:
- [File nào, sửa gì]
```

---

## Docs Gate 2 Tầng (Tóm tắt)

**Tầng 1 — CỨNG** (thiếu = KHÔNG commit):
Function signature, data flow, CLI interface, config schema,
data structure, module mới, runtime behavior.

**Tầng 2 — MỀM** (không block commit):
Tên biến, error message, code style.

Chi tiết: `docs/RULES.md`

---

## Language & Communication

- LUÔN nói tiếng Việt với Sếp
- Comment trong code: tiếng Anh
- Commit message: tiếng Anh, conventional commits

---

## Tech Stack

- **Language:** Rust (edition 2024)
- **CLI:** clap 4.x (derive)
- **Config:** toml + serde (cho .docs-gate.toml)
- **Regex:** regex crate
- **Testing:** `#[cfg(test)]`

---

## Git Workflow

```bash
git checkout main && git pull
git checkout -b feat/{phieu-id}-{tên-ngắn}
# Code, test → Commit Sequence
git push origin feat/{phieu-id}-{tên-ngắn}
```

**1 phiếu = 1 branch.**

---

## Khi nào PHẢI đọc docs/RULES.md

| Trigger | Đọc section nào |
|---------|----------------|
| Nhận phiếu `destructive` | Toàn bộ RULES.md |
| Nhận phiếu `mutating` | Phiếu Classification + Isolation |
| Lần đầu vào dự án | Đọc hết 1 lần |
| Session dài >10 phiếu | Context Management |
| Phiếu có sai lệch | Docs Gate Chi Tiết |

---

## Gotchas

- **Glob scan false positive:** Khi scan `*.md` trong directory, luôn xét có file nào KHÔNG phải target (TEMPLATE.md, README.md, etc.) sẽ bị bắt nhầm. Dùng `exclude_files` config.

---

## Phase hiện tại

Phase 1 (MVP CLI) ✅ HOÀN THÀNH
Phase 2 (Discovery Report + Ticket Check) ✅ HOÀN THÀNH
