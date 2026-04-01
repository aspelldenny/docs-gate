# RULES.md — Enforcement Chi Tiết

> File này chứa chi tiết các luật enforcement.
> Thợ đọc khi cần tra cứu, KHÔNG cần đọc mỗi session.
> Core rules nằm ở CLAUDE.md — luôn đọc trước.

---

## Phiếu Classification — READ-ONLY vs MUTATING

**Khi nhận phiếu, thợ PHẢI classify trước khi làm:**

| Type | Nghĩa | Ví dụ | Rule |
|------|--------|-------|------|
| `read-only` | Không đổi DB, schema, config, public API | Thêm GET endpoint, fix UI, thêm test | Làm song song OK, không cần confirm thêm |
| `mutating` | Đổi DB schema, data flow, public API, config | Migration, refactor service, đổi route | Làm xong + merge trước khi bắt phiếu tiếp |
| `destructive` | Xóa data, xóa module, đổi deploy | Drop table, remove service, đổi Docker | HARD STOP — hỏi Sếp confirm TRƯỚC KHI bắt đầu |

Phiếu không ghi type → thợ tự classify, báo Sếp trước khi bắt đầu.

---

## Context Management — Chống Mất Trí Nhớ

```
1. Mỗi 3-5 phiếu HOẶC khi context nặng:
   → Tự compact: "Tóm tắt: đã làm phiếu #X, #Y.
     Files sửa: [list]. Docs update: [list].
     Context cần giữ cho phiếu tiếp: [list]."

2. Bắt đầu quên context:
   → BÁO SẾP: "Em đang mất context, cần session mới"
   → KHÔNG đoán, KHÔNG giả vờ nhớ

3. Session quá dài (>20 phiếu hoặc >2h):
   → Đề xuất: "Nên mở session mới"
   → Tóm tắt state hiện tại trước khi kết thúc

4. Session mới, đọc lại theo thứ tự:
   CLAUDE.md → ARCHITECTURE.md → CHANGELOG.md (5 entries cuối) → ticket/
```

---

## Isolation — 1 Phiếu = 1 Branch

```
1. Mỗi phiếu = 1 branch: feat/{phieu-id}-{tên-ngắn}

2. KHÔNG làm 2 phiếu trên cùng 1 branch

3. KHÔNG commit code phiếu #2 lên branch phiếu #1

4. Branch merge vào main SAU KHI:
   - Sếp nghiệm thu pass
   - Docs Gate Tầng 1 pass
   - Discovery Report có

5. Phiếu #2 phụ thuộc phiếu #1:
   → #1 merge trước
   → Tạo branch #2 từ main mới
   → KHÔNG branch từ branch
```

---

## Docs Gate Chi Tiết

### Tầng 1 — CỨNG (thiếu = KHÔNG commit)

| Thay đổi gì | Ghi vào đâu | Ví dụ |
|-------------|-------------|-------|
| Function signature | ARCHITECTURE.md Section 7 | `handle_query nhận 10 params` |
| Lock type + concurrency model | ARCHITECTURE.md Section 7 | `filter dùng RwLock` |
| Data flow thứ tự | ARCHITECTURE.md Section 3 + 7 | `parse → filter → cache` |
| Config options + defaults | ARCHITECTURE.md Section 4 | `cache_size = 10000` |
| Cross-module interaction mới | ARCHITECTURE.md Section 9 | `CNAME check cần filter lock` |
| Data structure thay đổi | ARCHITECTURE.md Section 7 | `HashMap → LruCache` |
| Module mới / file mới | ARCHITECTURE.md Section 1 + 2 | Module map + API |
| Runtime behavior | ARCHITECTURE.md Section 8 | `Dashboard suppress console` |

### Tầng 2 — MỀM (không block commit)

| Thay đổi gì | Cần ghi? |
|-------------|----------|
| Tên biến local | Không |
| Error message wording | Không |
| Code formatting | Không |
| Comment detail | Không |

### Bảng mapping tổng hợp

| Thay đổi | File | Tầng |
|----------|------|------|
| Bất kỳ thay đổi code nào | CHANGELOG.md | 1 |
| Module/dependency mới | PROJECT.md | 1 |
| Public function mới/sửa | ARCHITECTURE.md Section 2 | 1 |
| Internal function quan trọng | ARCHITECTURE.md Section 7 | 1 |
| Lock type, data structure, algorithm | ARCHITECTURE.md Section 7 | 1 |
| Runtime behavior | ARCHITECTURE.md Section 8 | 1 |
| Cross-module interaction | ARCHITECTURE.md Section 9 | 1 |
| Config option mới | ARCHITECTURE.md Section 4 | 1 |
| Data flow | ARCHITECTURE.md Section 3 | 1 |
| Gotcha mới | CLAUDE.md Gotchas | 1 |
| Variable naming, code style | Tùy | 2 |

### Khi phiếu có sai lệch

**Sai Tầng 2:** Thợ TỰ QUYẾT, ghi Discovery Report.

**Sai Tầng 1:** Thợ implement theo code thật (KHÔNG theo phiếu sai),
BẮT BUỘC update ARCHITECTURE.md, ghi Discovery Report.
Nếu sai lệch đổi scope → DỪNG hỏi Sếp.

---

## ARCHITECTURE.md — 9 Sections Bắt Buộc

| # | Section | Ghi cái gì |
|---|---------|-----------|
| 1 | Module Map | Cây file, mỗi file 1 dòng mô tả |
| 2 | Public API | Signature + description |
| 3 | Data Flow | ASCII diagram data đi qua hệ thống |
| 4 | Config | Mọi config option + default |
| 5 | Dependencies | Package + version + dùng cho gì |
| 6 | Error Handling | Mỗi loại lỗi xử lý thế nào |
| 7 | Implementation Notes | Data structure, lock type, algorithm, trade-offs |
| 8 | Runtime Behavior | Process model, output, signals, startup/shutdown |
| 9 | Known Constraints | Cross-module issues, limitations |

**Section 7, 8, 9 là BẮT BUỘC. Thiếu = task CHƯA XONG.**

### Section 7 ghi gì:
- Data structure: HashMap? HashSet? Array? — GHI RÕ
- Concurrency model: Mutex? Lock? Atomic? — GHI RÕ
- Algorithm + complexity: O(1)? O(n)? — GHI RÕ
- Trade-off: GHI 1 DÒNG LÝ DO
- KHÔNG handle gì: GHI RÕ

### Section 9 format:
```
### [Tên ngắn]
- **Modules liên quan:** A ↔ B
- **Vấn đề:** 1-2 câu
- **Xử lý hiện tại:** đã fix / workaround / chưa fix
- **Nếu chưa fix:** impact + khi nào fix
```

---

## Flow Hoàn Chỉnh Khi Xong Phiếu

```
Nhận phiếu
  → Classify (read-only / mutating / destructive)
  → Check Hard Stops
  → Code + test
  → Docs Gate Tầng 1
  → Discovery Report
  → Commit Sequence
  → Báo Sếp "xong"
  → Sếp nghiệm thu
  → Sếp move ticket/ → archive/
```

Thợ KHÔNG tự move phiếu vào archive.
