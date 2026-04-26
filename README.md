# docs-gate

**Docs compliance checks before every commit.**

[![CI](https://github.com/aspelldenny/docs-gate/actions/workflows/ci.yml/badge.svg)](https://github.com/aspelldenny/docs-gate/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A Rust CLI that enforces documentation standards in projects using structured
markdown. Checks that your CHANGELOG.md has recent entries, your ARCHITECTURE.md
has required sections, your tickets have valid types, and your Discovery Reports
follow the expected format. Single binary, zero runtime dependencies.

---

## Install

```bash
cargo install --git https://github.com/aspelldenny/docs-gate
```

Or from source:

```bash
git clone https://github.com/aspelldenny/docs-gate.git
cd docs-gate
cargo install --path .
```

## Quick start

```bash
# Run core checks (changelog + architecture)
docs-gate

# Run all checks including ticket type validation
docs-gate --all

# Auto-generate config from your project layout
docs-gate init
```

## Commands

| Command | Description |
|---------|-------------|
| *(default)* | Run core checks (changelog + architecture) |
| `check-discovery <file>` | Validate Discovery Report format in a file |
| `serve` | Start MCP server on stdio transport |
| `init` | Scan project and generate `.docs-gate.toml` |

## Flags

| Flag | Description |
|------|-------------|
| `--all` | Include ticket type checks |
| `--watch` | Re-run checks on file changes (Ctrl+C to exit) |
| `--verbose` | Show passing checks in output |
| `--config <path>` | Path to config file |

## Checks

| Check | What it verifies |
|-------|-----------------|
| `changelog` | CHANGELOG.md has a recent entry (date + content) |
| `architecture-section-count` | ARCHITECTURE.md has the required number of sections |
| `architecture-section-{N}` | Specified sections are non-empty |
| `ticket-type` | Ticket files contain a valid type classification |
| `discovery` | Discovery Report follows the expected 4-section format |

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | One or more checks failed |
| 2 | Usage error |

## Configuration

Optional `.docs-gate.toml` in project root. Run `docs-gate init` to generate one automatically.

```toml
docs_dir = "docs"
changelog = "CHANGELOG.md"
architecture = "ARCHITECTURE.md"
required_sections = 9
required_non_empty = [7, 8, 9]
changelog_max_age_days = 1

[ticket]
ticket_dir = "docs/ticket"
valid_types = ["read-only", "mutating", "destructive"]
exclude_files = ["TEMPLATE.md"]

# Optional: enforce structure on additional doc files beyond ARCHITECTURE.md.
# Each [[doc_structure]] entry runs the same section-count + required-non-empty
# checks against the named file under docs_dir.
[[doc_structure]]
file = "TEST_CASES.md"
required_sections = 3
required_non_empty = [1, 2, 3]

[[doc_structure]]
file = "AUDIT_PROTOCOL.md"
required_sections = 6
```

All fields are optional. Without a config file, sensible defaults apply.

## MCP server

docs-gate can run as an [MCP](https://modelcontextprotocol.io/) server,
letting AI assistants call docs checks as tools.

```bash
docs-gate serve
```

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "docs-gate": {
      "command": "docs-gate",
      "args": ["serve"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

Available MCP tools: `check_changelog`, `check_architecture`, `check_discovery`, `check_all`.

## Git hook

```bash
# .git/hooks/pre-commit
#!/bin/sh
docs-gate
```

## Requirements

- Rust 1.85+ (edition 2024)

## Status

v0.1 -- functional and tested, used in production workflows. API may evolve.

## Contributing

1. Fork the repo
2. Create a feature branch
3. Run `cargo test && cargo clippy -- -D warnings` before submitting
4. Open a PR against `main`

## License

MIT -- see [LICENSE](LICENSE).
