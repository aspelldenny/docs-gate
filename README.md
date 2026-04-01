# docs-gate

CLI tool to check docs compliance before commit. Designed for projects using structured markdown documentation (CHANGELOG.md + ARCHITECTURE.md).

Written in Rust. Single binary, zero runtime dependencies.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Run checks with defaults (looks for docs/ directory)
docs-gate

# Show all results including passes
docs-gate --verbose

# Use custom config file
docs-gate --config path/to/.docs-gate.toml
```

### Output

```
✅ PASS: changelog
✅ PASS: architecture-section-count
✅ PASS: architecture-section-7
✅ PASS: architecture-section-8
✅ PASS: architecture-section-9

✅ All checks passed (5/5)
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | All checks passed |
| 1 | One or more checks failed |

### Extended Checks

```bash
# Run all checks including ticket type validation
docs-gate --all

# Check a specific file for Discovery Report format
docs-gate check-discovery path/to/report.md
```

### Watch Mode

Re-run checks automatically when files change:

```bash
# Watch docs directory, re-run on changes
docs-gate --watch

# Watch docs + ticket directory
docs-gate --watch --all
```

Press `Ctrl+C` to exit watch mode.

### MCP Server Mode

Start an [MCP](https://modelcontextprotocol.io/) server on stdio transport, allowing AI assistants to run docs checks:

```bash
docs-gate serve
```

#### Claude Desktop Configuration

Add to your `claude_desktop_config.json`:

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

See `examples/claude_desktop_config.json` for a ready-to-use template.

#### Available MCP Tools

| Tool | Parameters | Description |
|------|-----------|-------------|
| `check_changelog` | `docs_dir` (optional) | Check CHANGELOG.md has a recent entry |
| `check_architecture` | `docs_dir` (optional) | Check ARCHITECTURE.md 9 sections + non-empty 7,8,9 |
| `check_discovery` | `file_path` (required) | Check Discovery Report format in a file |
| `check_all` | `docs_dir` (optional) | Run all checks (changelog + architecture + tickets) |

### Use as git hook

```bash
# .git/hooks/pre-commit
#!/bin/sh
docs-gate
```

## Checks

| Check | What it does |
|-------|-------------|
| changelog | Verify CHANGELOG.md has a recent entry (date within N days, with content) |
| architecture-section-count | Verify ARCHITECTURE.md has required number of sections |
| architecture-section-{N} | Verify required sections are not empty |

## Config

Optional `.docs-gate.toml` in project root:

```toml
docs_dir = "docs"                # Directory containing docs (default: "docs")
changelog = "CHANGELOG.md"       # Changelog filename (default: "CHANGELOG.md")
architecture = "ARCHITECTURE.md" # Architecture filename (default: "ARCHITECTURE.md")
required_sections = 9            # Required section count (default: 9)
required_non_empty = [7, 8, 9]   # Sections that must have content (default: [7, 8, 9])
changelog_max_age_days = 1       # Max age of latest changelog entry in days (default: 1)
```

All options are optional. Without a config file, sensible defaults are used.

## License

MIT
