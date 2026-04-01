# PROJECT.md — docs-gate

## Vision

CLI tool kiểm tra docs compliance trước khi commit.
Dùng cho mọi dự án theo hệ thống thợ-thầu (CLAUDE.md + ARCHITECTURE.md 9 sections + CHANGELOG).
Chạy như git hook hoặc standalone. Viết bằng Rust, single binary, zero runtime dependency.

## Tech Stack

- **Language:** Rust (edition 2024)
- **CLI:** clap 4.x (derive feature)
- **Config:** toml + serde (.docs-gate.toml per-project)
- **Regex:** regex crate (parse markdown headings, detect content)
- **Testing:** built-in `#[cfg(test)]`
- **Distribution:** Single binary (cargo install hoặc copy)

## Features

| Feature | Status | Phase |
|---------|--------|-------|
| Check CHANGELOG.md updated | ✅ | 1 |
| Check ARCHITECTURE.md 9 sections | ✅ | 1 |
| Check ARCHITECTURE.md Section 7,8,9 not empty | ✅ | 1 |
| Per-project config (.docs-gate.toml) | ✅ | 1 |
| Exit code 0/1 (dùng làm git hook) | ✅ | 1 |
| Human-readable output (pass/fail + lý do) | ✅ | 1 |
| Check Discovery Report format | ✅ | 2 |
| Check phiếu có type classification | ✅ | 2 |
| MCP server mode | ✅ | 3 |
| Watch mode (re-check on file change) | ✅ | 3 |

## Target Users

- Solo dev dùng hệ thống thợ-thầu với AI (Claude Code + Claude Web)
- Bất kỳ team nào dùng markdown docs as source of truth

## Roadmap

| Phase | Mục tiêu | Status |
|-------|----------|--------|
| 1 | MVP CLI: check CHANGELOG + ARCHITECTURE + config | ✅ |
| 2 | Extended checks: Discovery Report, phiếu format | ✅ |
| 3 | MCP server + watch mode | ✅ |
| 4 | Open source release (README, CI/CD, cargo install) | 📋 |

## Constraints

- Single binary, no runtime dependencies
- Config optional — sensible defaults work without .docs-gate.toml
- Async runtime: tokio current_thread (cho watch mode + signal handling, checks vẫn sync)
- Output phải readable cho cả human và CI/CD parsing
