# Phase 1: MVP CLI

**Type:** `mutating` (tạo toàn bộ source code từ đầu)

## Tổng quan

Build CLI tool `docs-gate` kiểm tra docs compliance. Chạy 1 lệnh → check CHANGELOG + ARCHITECTURE → pass/fail.

## Phiếu #1-1: Project skeleton + config

**Type:** `mutating`

**Nhiệm vụ:** Setup Cargo.toml dependencies, tạo module structure, implement config loading.

**Luật chơi:**
- Dùng clap derive cho CLI args
- Config file optional, defaults phải work
- Tất cả theo ARCHITECTURE.md Section 1 (Module Map)

**Kịch bản lỗi:**
- .docs-gate.toml không tồn tại → dùng defaults, không error
- .docs-gate.toml parse lỗi → stderr warning, dùng defaults

**Nghiệm thu:**
- [ ] `cargo build --release` zero warnings
- [ ] `docs-gate --help` hiện usage
- [ ] `docs-gate` chạy ở thư mục không có config → dùng defaults
- [ ] `docs-gate` chạy ở thư mục có .docs-gate.toml → load config

**Assumptions:**
- CLI args: `docs-gate [--config path] [--verbose]`
- Module structure theo ARCHITECTURE.md Section 1

---

## Phiếu #1-2: CHANGELOG check

**Type:** `read-only`

**Nhiệm vụ:** Implement check_changelog() — verify CHANGELOG.md có entry gần đây.

**Luật chơi:**
- Detect `## ` headings
- Tìm date YYYY-MM-DD trong heading hoặc dòng đầu
- "Recent" = trong changelog_max_age_days (default 1)
- Entry phải có content (>= 1 non-empty non-comment line)

**Kịch bản lỗi:**
- File không tồn tại → Fail("File not found")
- File trống → Fail("No entries found")
- Entry có heading nhưng no content → Fail("Entry empty")
- Date > max_age_days → Fail("Last entry too old: {date}")

**Nghiệm thu:**
- [ ] File có entry hôm nay → Pass
- [ ] File có entry hôm qua (max_age=1) → Pass
- [ ] File có entry 3 ngày trước (max_age=1) → Fail
- [ ] File không tồn tại → Fail
- [ ] File trống → Fail
- [ ] Tests: >= 5 unit tests

**Assumptions:**
- Changelog format: `## [label] Title — YYYY-MM-DD`
- Date regex: `\d{4}-\d{2}-\d{2}`

---

## Phiếu #1-3: ARCHITECTURE check

**Type:** `read-only`

**Nhiệm vụ:** Implement check_architecture() — verify 9 sections + 7,8,9 non-empty.

**Luật chơi:**
- Detect `## N.` pattern (N = section number)
- Count unique section numbers
- Check sections trong required_non_empty có content
- Content = lines between 2 headings, exclude comments (`<!-- -->`), whitespace

**Kịch bản lỗi:**
- File không tồn tại → Fail
- Chỉ có 5 sections → Fail("Found 5/9 sections")
- Section 7 empty (chỉ có template comments) → Fail("Section 7 empty")
- Section 9 missing → Fail("Section 9 missing")

**Nghiệm thu:**
- [ ] File đầy đủ 9 sections, 7/8/9 có content → Pass
- [ ] File thiếu section 8 → Fail
- [ ] Section 7 chỉ có `<!-- comments -->` → Fail (empty)
- [ ] File không tồn tại → Fail
- [ ] Tests: >= 5 unit tests

**Assumptions:**
- Section heading regex: `^## \d+\.`
- Comments: `<!--` to `-->`

---

## Phiếu #1-4: Output formatting + integration

**Type:** `mutating`

**Nhiệm vụ:** Implement output.rs, wire everything together trong main.rs.

**Luật chơi:**
- stdout: human-readable results
- Exit code: 0 all pass, 1 any fail
- --verbose flag: show details cho Pass results too

**Kịch bản lỗi:**
- Tất cả check pass → exit 0
- 1 check fail → exit 1, list all results (pass + fail)

**Nghiệm thu:**
- [ ] Chạy trong repo có docs đầy đủ → all ✅, exit 0
- [ ] Chạy trong repo thiếu CHANGELOG → ❌ FAIL, exit 1
- [ ] Output format: `✅ PASS: changelog` / `❌ FAIL: architecture — Section 7 empty`
- [ ] `echo $?` sau lệnh → 0 hoặc 1
- [ ] Integration test: build binary, run on test fixtures
- [ ] README.md basic usage

**Assumptions:**
- Output theo ARCHITECTURE.md Section 7 (output.rs)
