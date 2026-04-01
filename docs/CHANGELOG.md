# CHANGELOG

## [#1-4] Output formatting + integration tests — 2026-04-01
- 5 unit tests for output.rs: format verbose/non-verbose, failure output, exit code 0/1
- 5 integration tests: all pass exit 0, missing changelog exit 1, missing architecture exit 1, verbose shows pass, custom config
- Total: 29 tests (24 unit + 5 integration)

## [#1-3] ARCHITECTURE check unit tests — 2026-04-01
- 7 unit tests for check_architecture(): full 9 sections pass, file not found fail, missing sections fail, section 7 empty (comments only) fail, section 9 missing fail, section 8 whitespace-only fail, custom required_sections pass

## [#1-2] CHANGELOG check unit tests — 2026-04-01
- 8 unit tests for check_changelog(): pass today, pass yesterday, fail too old, fail not found, fail empty, fail no heading, fail empty content, fail no date

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
