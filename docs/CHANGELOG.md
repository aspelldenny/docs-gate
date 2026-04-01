# CHANGELOG

## [#4-1] Open source release — CI/CD, packaging, docs polish — 2026-04-01
- Cargo.toml: added package metadata (description, license, repository, keywords, categories, readme)
- LICENSE: MIT license file created
- CI/CD: .github/workflows/ci.yml — 4 jobs (check, test, build-release, release)
  - Lint: fmt + clippy on ubuntu
  - Test: 3 OS matrix (ubuntu, macos, windows)
  - Build: 4 targets (linux x86_64, macos x86_64/aarch64, windows x86_64)
  - Release: auto GitHub Release on tag v*
- README: badges, ticket config section, contributing section, install options
- NO code changes in src/

## [#3-3] MCP end-to-end testing + documentation — 2026-04-01
- 9 new MCP integration tests: initialize, tools/list, 4 tool calls (pass+fail), shutdown
- README.md: added MCP Server Mode, Watch Mode, Claude Desktop config, Available MCP Tools
- examples/claude_desktop_config.json: ready-to-use Claude Desktop config template
- Fixed: ServerHandler now properly delegates list_tools/call_tool to tool_router
- Total: 68 tests (52 unit + 11 CLI integration + 9 MCP integration) — note: count includes 4 server unit tests added in #3-2
- NO new code in src/ — only tests + docs

## [#3-2] MCP server skeleton + stdio transport — 2026-04-01
- New module: mcp/ (mod.rs, server.rs, tools.rs) — MCP server via rmcp SDK
- CLI: `docs-gate serve` subcommand — starts MCP server on stdio transport
- 4 MCP tools: check_changelog, check_architecture, check_discovery, check_all
- Tools call existing sync check functions directly — no logic duplication
- DocsGateServer with ServerHandler impl, tool_router macro for tool routing
- CheckResult/CheckStatus now derive Serialize + JsonSchema for MCP responses
- Dependencies: rmcp 0.8.x, schemars 1.x, serde_json 1.x
- 4 new unit tests (server info, tool router, config resolution)
- Total: 59 tests (52 unit + 11 integration) — note: serde_json needed for JSON serialization

## [#3-1] Async migration + Watch mode — 2026-04-01
- Migrated main.rs to tokio async runtime (#[tokio::main(flavor = "current_thread")])
- New module: watch.rs — watch mode with file system monitoring
- CLI: --watch flag for re-running checks on file changes
- --watch + --all watches both docs_dir and ticket_dir
- --watch + check-discovery → error exit 2 (not supported)
- Debounce 500ms, terminal clear + timestamp before each re-run
- Ctrl+C → clean exit with last check exit code
- Dependencies: tokio 1.x (rt, macros, signal, sync, time), notify 8.x, libc 0.2 (dev)
- 4 new unit tests (watch.rs), 2 new integration tests (watch error + SIGINT)
- Total: 55 tests (48 unit + 11 integration) — note: libc added to dev-deps for SIGINT test

## [#2-1] Discovery Report check + Ticket type classification — 2026-04-01
- New module: checks/discovery.rs — check Discovery Report format (4 required sections)
- New module: checks/ticket.rs — scan ticket dir for Type declarations
- CLI: `check-discovery <file>` subcommand, `--all` flag for ticket checks
- Config: nested `[ticket]` section with ticket_dir, valid_types, exclude_files
- Config backward compatible: flat keys unchanged, new keys under [ticket]
- run_all_checks_extended() for --all mode, run_all_checks() unchanged
- 16 new unit tests (7 discovery + 7 ticket + 2 config), 4 new integration tests
- Total: 49 tests (40 unit + 9 integration)

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
