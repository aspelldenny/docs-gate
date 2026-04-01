# CHANGELOG

## [#1-1] Project skeleton + config — 2026-04-01
- Dependencies: clap 4.x, serde 1.x, toml 0.8.x, regex 1.x, chrono 0.4.x, tempfile 3.x (dev)
- Module structure: main.rs, config.rs, checks/mod.rs, checks/changelog.rs, checks/architecture.rs, output.rs
- Config loading: .docs-gate.toml optional, defaults work, parse error → stderr warning + defaults
- CLI: --config, --verbose flags via clap derive
- CheckStatus enum: Pass, Fail(String), Warn(String)
- All checks wired: changelog + architecture (section count + non-empty 7,8,9)
- Output: human-readable ✅/❌ + summary + exit code 0/1

## [Setup] Project init — 2026-04-01
- Files: CLAUDE.md, docs/PROJECT.md, docs/ARCHITECTURE.md, docs/RULES.md, docs/CHANGELOG.md
- Dependencies: (none yet)
- Cargo init, template docs populated
