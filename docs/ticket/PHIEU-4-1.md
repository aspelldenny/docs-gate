# Phiếu #4-1: Open source release — CI/CD, packaging, docs polish

**Type:** `mutating`

**Khối lượng:** Vừa (1-3 ngày). Gộp CI/CD + packaging + docs vì cùng mục tiêu release, không có risk cao, thợ làm 1 lượt được.

---

## Tổng quan

Chuẩn bị dự án cho open source release:
1. Cargo.toml metadata cho `cargo install` / crates.io publish
2. LICENSE file
3. GitHub Actions CI/CD pipeline (build, test, clippy, cross-compile releases)
4. README polish (badges, config examples, contributing section)

---

## Nhiệm vụ

### A. Cargo.toml metadata

Thêm/sửa fields trong `[package]`:

```toml
[package]
name = "docs-gate"
version = "0.1.0"
edition = "2024"
description = "CLI tool to check docs compliance before commit — CHANGELOG, ARCHITECTURE, Discovery Reports"
license = "MIT"
repository = "https://github.com/[owner]/docs-gate"
homepage = "https://github.com/[owner]/docs-gate"
keywords = ["documentation", "linting", "changelog", "architecture", "cli"]
categories = ["command-line-utilities", "development-tools"]
readme = "README.md"
```

**Thợ verify:** Cargo.toml hiện tại có những fields nào trong `[package]`. Chỉ thêm fields thiếu, KHÔNG đổi fields đã có (trừ khi trống/placeholder).

**`repository` và `homepage`:** Dùng placeholder `https://github.com/OWNER/docs-gate`. Sếp sẽ thay bằng URL thật trước khi publish. Ghi comment `# TODO: replace OWNER` cạnh dòng đó.

### B. LICENSE file

Tạo file `LICENSE` ở project root. Nội dung: MIT License standard text.

**Thợ dùng template MIT chuẩn.** Year: 2026. Copyright holder: placeholder `docs-gate contributors` (Sếp sẽ sửa nếu cần).

### C. GitHub Actions CI/CD

Tạo `.github/workflows/ci.yml`:

**Triggers:**
- `push` to `main`
- `pull_request` to `main`

**Jobs:**

**Job 1: `check`** (chạy nhanh, fail early)
- Matrix: `ubuntu-latest` only (đủ cho lint)
- Steps: checkout → setup rust (stable) → `cargo fmt -- --check` → `cargo clippy -- -D warnings`

**Job 2: `test`** (depends on `check`)
- Matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`
- Steps: checkout → setup rust (stable) → `cargo test`

**Job 3: `build-release`** (depends on `test`, only on `push` to `main` OR tag `v*`)
- Matrix:
  - `ubuntu-latest` → target `x86_64-unknown-linux-gnu`
  - `macos-latest` → target `x86_64-apple-darwin`
  - `macos-latest` → target `aarch64-apple-darwin`
  - `windows-latest` → target `x86_64-pc-windows-msvc`
- Steps: checkout → setup rust (stable) → add target → `cargo build --release --target {target}`
- Upload artifact: binary file cho mỗi target

**Job 4: `release`** (depends on `build-release`, only on tag `v*`)
- Download artifacts từ `build-release`
- Tạo GitHub Release với binaries attached
- Dùng `softprops/action-gh-release` action

**Luật:**
- KHÔNG dùng nightly rust — stable only
- KHÔNG cache phức tạp — `actions/cache` cho `~/.cargo/registry` + `target/` là đủ
- Rust toolchain setup dùng `dtolnay/rust-toolchain` action

### D. README polish

Sửa `README.md` hiện tại. KHÔNG viết lại từ đầu — thêm/sửa sections.

**Thêm ở đầu file (sau heading `# docs-gate`):**
- Badges: CI status, crates.io version, license
- Format: `[![CI](url)](url)` — dùng placeholder URLs, thợ ghi `<!-- TODO: update badge URLs after repo is public -->`

**Thêm section `## Contributing`** (cuối file, trước License section):
```
## Contributing

Contributions welcome! Please:
1. Fork the repo
2. Create a feature branch
3. Run `cargo test && cargo clippy -- -D warnings` before submitting
4. Open a PR against `main`
```

**Thêm section `## Ticket Config`** (sau section Config hiện tại):
- Document `[ticket]` config section (ticket_dir, valid_types, exclude_files)
- Hiện tại README chưa document ticket config — chỉ có flat config

**Sửa Install section:**
- Thêm `cargo install docs-gate` option (cho khi đã publish lên crates.io)
- Giữ `cargo install --path .` cho local build
- Ghi note: `cargo install docs-gate` sẽ available sau khi publish

**KHÔNG đổi:** Sections Usage, Checks, MCP Server Mode, Watch Mode — đã đủ từ #3-3.

### E. .gitignore review

**Thợ verify:** `.gitignore` hiện tại có cover `target/`, `.docs-gate.toml` (optional — user config, không nên commit). Nếu thiếu → thêm. Nếu đủ → skip.

**Lưu ý:** `.docs-gate.toml` KHÔNG nên nằm trong .gitignore vì mỗi project có config riêng, user muốn commit config của project mình. Chỉ ignore nếu nó là config cá nhân.

---

## Files cần tạo/sửa

| File | Action | Gì |
|------|--------|----|
| `Cargo.toml` | Sửa | Thêm package metadata |
| `LICENSE` | Tạo mới | MIT license text |
| `.github/workflows/ci.yml` | Tạo mới | CI/CD pipeline |
| `README.md` | Sửa | Badges, ticket config, contributing, install update |
| `.gitignore` | Verify/sửa | Ensure target/ covered |

---

## Luật chơi

- KHÔNG đổi bất kỳ code nào trong `src/` — phiếu này chỉ đụng metadata + docs + CI
- KHÔNG thêm dependency mới
- KHÔNG đổi CLI interface, config schema, hay behavior
- Placeholder URLs cho GitHub repo — Sếp sẽ thay sau
- CI config phải work ngay khi push lên GitHub (trừ badge URLs)

---

## Kịch bản lỗi

- Cargo.toml đã có một số metadata fields → chỉ thêm fields thiếu, KHÔNG overwrite
- `.gitignore` không tồn tại → tạo mới với `target/` entry
- GitHub Actions syntax sai → thợ test bằng cách đọc kỹ actions docs, em tin thợ biết GitHub Actions

---

## Nghiệm thu

### Cargo.toml:
- [ ] `cargo build --release` vẫn zero warnings sau khi thêm metadata
- [ ] `cargo package --list` chạy thành công, hiện danh sách files sẽ publish
- [ ] Fields: description, license, repository, keywords, categories đều có

### LICENSE:
- [ ] File `LICENSE` tồn tại ở project root
- [ ] Nội dung là MIT license chuẩn

### CI/CD:
- [ ] `.github/workflows/ci.yml` tồn tại
- [ ] YAML syntax valid (thợ verify bằng cách parse mentally hoặc dùng online YAML validator)
- [ ] 4 jobs: check, test, build-release, release
- [ ] Test matrix: 3 OS (ubuntu, macos, windows)
- [ ] Build matrix: 4 targets (linux x86_64, macos x86_64, macos aarch64, windows x86_64)
- [ ] Release job chỉ trigger on tag `v*`

### README:
- [ ] Badges section ở đầu file
- [ ] `[ticket]` config section documented
- [ ] Contributing section có
- [ ] Install section có cả `cargo install docs-gate` và `cargo install --path .`
- [ ] Existing content (Usage, MCP, Watch) KHÔNG bị đổi

### Build:
- [ ] `cargo build --release` zero warnings
- [ ] `cargo test` all pass (68 tests)
- [ ] `cargo clippy -- -D warnings` clean

---

## Assumptions

- Cargo.toml hiện chỉ có `name`, `version`, `edition` trong [package] — thợ verify và chỉ thêm fields thiếu
- `.gitignore` có thể đã tồn tại hoặc chưa — thợ verify
- Project sẽ host trên GitHub — CI dùng GitHub Actions
- License là MIT (theo README.md hiện tại đã ghi "MIT")
- ARCHITECTURE.md Section 1 KHÔNG cần update vì phiếu này không thêm module code nào

---

## Docs cần update sau khi xong

- CHANGELOG.md: ghi entry (files tạo/sửa, không có code changes)
- PROJECT.md: update Phase 4 status → ✅ (nếu Sếp confirm đây là toàn bộ Phase 4)
- CLAUDE.md: update "Phase hiện tại" → Phase 4 ✅
- ARCHITECTURE.md: KHÔNG cần update — phiếu này không đổi code, không đổi module map, không đổi API
