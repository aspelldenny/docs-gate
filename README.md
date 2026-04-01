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
