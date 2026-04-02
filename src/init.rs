use std::collections::HashMap;
use std::path::Path;

use crate::config::{ArchitectureConfig, Config, TicketConfig};

/// Scan a project directory and generate a Config that matches its structure.
pub fn scan_project(root: &Path) -> Config {
    let docs_dir = detect_docs_dir(root);
    let changelog = detect_changelog(&docs_dir);
    let architecture = detect_architecture(&docs_dir);
    let mut ticket = detect_tickets(&docs_dir);

    // Convert ticket_dir to relative path
    if let Ok(rel) = ticket.ticket_dir.strip_prefix(root) {
        ticket.ticket_dir = rel.to_path_buf();
    }

    Config {
        docs_dir: docs_dir
            .strip_prefix(root)
            .unwrap_or(&docs_dir)
            .to_path_buf(),
        changelog,
        changelog_max_age_days: 1,
        architecture,
        ticket,
    }
}

/// Write config to .docs-gate.toml, return the serialized string.
pub fn write_config(root: &Path, config: &Config) -> std::io::Result<String> {
    let content = toml::to_string_pretty(config).expect("Config should be serializable");
    let path = root.join(".docs-gate.toml");
    std::fs::write(&path, &content)?;
    Ok(content)
}

fn detect_docs_dir(root: &Path) -> std::path::PathBuf {
    let candidates = ["docs", "doc", "documentation"];
    for c in candidates {
        let p = root.join(c);
        if p.is_dir() {
            return p;
        }
    }
    // Fallback: use root itself
    root.to_path_buf()
}

fn detect_changelog(docs_dir: &Path) -> String {
    let candidates = ["CHANGELOG.md", "changelog.md", "CHANGES.md"];
    for c in candidates {
        if docs_dir.join(c).exists() {
            return c.to_string();
        }
    }
    // Check parent (root) too
    if let Some(parent) = docs_dir.parent() {
        for c in candidates {
            if parent.join(c).exists() {
                return c.to_string();
            }
        }
    }
    String::from("CHANGELOG.md")
}

fn detect_architecture(docs_dir: &Path) -> ArchitectureConfig {
    let candidates = [
        "ARCHITECTURE.md",
        "architecture.md",
        "DESIGN.md",
        "SYSTEM_DESIGN.md",
    ];

    let re = regex::Regex::new(r"^## (\d+)\.").unwrap();
    for c in candidates {
        let path = docs_dir.join(c);
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let section_nums: std::collections::HashSet<usize> = content
                .lines()
                .filter_map(|line| re.captures(line))
                .filter_map(|cap| cap[1].parse().ok())
                .collect();

            let count = section_nums.len();
            let max_section = section_nums.iter().copied().max().unwrap_or(0);

            // Require non-empty for last 3 sections (or all if fewer than 3)
            let required_non_empty = if count >= 3 {
                ((max_section - 2)..=max_section).collect()
            } else {
                section_nums.iter().copied().collect()
            };

            return ArchitectureConfig {
                enabled: true,
                file: c.to_string(),
                required_sections: count,
                required_non_empty,
            };
        }
    }

    ArchitectureConfig {
        enabled: false,
        file: String::from("ARCHITECTURE.md"),
        required_sections: 0,
        required_non_empty: vec![],
    }
}

fn detect_tickets(docs_dir: &Path) -> TicketConfig {
    let candidates = ["ticket", "tickets", "phieu"];
    let mut ticket_dir = docs_dir.join("ticket");

    for c in candidates {
        let p = docs_dir.join(c);
        if p.is_dir() {
            ticket_dir = p;
            break;
        }
    }

    if !ticket_dir.exists() {
        return TicketConfig {
            ticket_dir: ticket_dir
                .parent()
                .map(|p| p.join("ticket"))
                .unwrap_or(ticket_dir),
            type_pattern: String::new(),
            valid_types: vec![],
            exclude_files: vec![],
        };
    }

    let entries: Vec<_> = std::fs::read_dir(&ticket_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .collect();

    // Detect exclude files (templates, READMEs)
    let exclude_keywords = ["template", "readme", "example", "sample"];
    let exclude_files: Vec<String> = entries
        .iter()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            exclude_keywords.iter().any(|kw| name.contains(kw))
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // Read non-excluded ticket files to detect type pattern
    let ticket_contents: Vec<String> = entries
        .iter()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            !exclude_files.contains(&name)
        })
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect();

    let (type_pattern, valid_types) = detect_type_pattern(&ticket_contents);

    TicketConfig {
        ticket_dir,
        type_pattern,
        valid_types,
        exclude_files,
    }
}

/// Scan ticket contents to find the most common bold-key-backtick pattern.
/// Returns (pattern, collected_values). Empty string = no pattern found.
fn detect_type_pattern(contents: &[String]) -> (String, Vec<String>) {
    if contents.is_empty() {
        return (String::new(), vec![]);
    }

    // Try common patterns: **Key:** `value`
    let patterns = [
        (r"\*\*Type:\*\*\s*`([^`]+)`", "Type"),
        (r"\*\*Loại:\*\*\s*`([^`]+)`", "Loại"),
        (r"\*\*Classification:\*\*\s*`([^`]+)`", "Classification"),
    ];

    let mut best_pattern: Option<(&str, Vec<String>)> = None;
    let mut best_count = 0;

    for (pattern, _label) in &patterns {
        let re = regex::Regex::new(pattern).unwrap();
        let mut values: Vec<String> = Vec::new();
        let mut match_count = 0;

        for content in contents {
            for cap in re.captures_iter(content) {
                values.push(cap[1].to_string());
                match_count += 1;
            }
        }

        if match_count > best_count {
            best_count = match_count;
            best_pattern = Some((pattern, values));
        }
    }

    // Also try generic pattern: **AnyKey:** `value`
    let generic_re = regex::Regex::new(r"\*\*([^*]+):\*\*\s*`([^`]+)`").unwrap();
    let mut key_counts: HashMap<String, Vec<String>> = HashMap::new();

    for content in contents {
        for cap in generic_re.captures_iter(content) {
            key_counts
                .entry(cap[1].to_string())
                .or_default()
                .push(cap[2].to_string());
        }
    }

    // Find the key that appears in most files with limited unique values (likely a type/category)
    // Filter out keys where values look like file paths, URLs, or other non-categorical data
    if let Some((key, values)) = key_counts
        .iter()
        .filter(|(key, vals)| {
            // Key name should be short and simple (e.g. "Type", "Loại", "Classification")
            // Skip keys with spaces (e.g. "Service test (macOS only)", "Type-check")
            let key_looks_like_type = key.len() <= 20 && !key.contains(' ') && !key.contains('(');
            let unique: std::collections::HashSet<_> = vals.iter().collect();
            let type_value_re = regex::Regex::new(r"^[a-zA-ZÀ-ỹ][a-zA-ZÀ-ỹ0-9_-]{0,28}$").unwrap();
            let is_categorical = vals.iter().all(|v| {
                // Type/category values: single word with letters/hyphens/underscores
                // e.g. "mutating", "read-only", "Hotfix", "Prompt-only", "Feature"
                // Reject anything with spaces (commands, descriptions)
                type_value_re.is_match(v)
            });
            key_looks_like_type
                && unique.len() <= 10
                && vals.len() >= contents.len() / 2
                && is_categorical
        })
        .max_by_key(|(_, vals)| vals.len())
    {
        let generic_count = values.len();
        if generic_count > best_count {
            let pattern = format!(r"\*\*{key}:\*\*\s*`([^`]+)`");
            let unique: std::collections::HashSet<_> = values.iter().collect();
            let mut sorted: Vec<String> = unique.into_iter().cloned().collect();
            sorted.sort();
            return (pattern, sorted);
        }
    }

    match best_pattern {
        Some((pattern, values)) => {
            let unique: std::collections::HashSet<_> = values.iter().collect();
            let mut sorted: Vec<String> = unique.into_iter().cloned().collect();
            sorted.sort();
            (pattern.to_string(), sorted)
        }
        None => (String::new(), vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn setup_project(dir: &Path) {
        let docs = dir.join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::create_dir_all(docs.join("ticket")).unwrap();

        // CHANGELOG.md
        let mut f = fs::File::create(docs.join("CHANGELOG.md")).unwrap();
        writeln!(f, "# Changelog\n\n## 2025-01-01\n- Init").unwrap();
    }

    #[test]
    fn test_scan_basic_project() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(dir.path());

        let config = scan_project(dir.path());
        assert_eq!(config.docs_dir, Path::new("docs"));
        assert_eq!(config.changelog, "CHANGELOG.md");
        assert!(!config.architecture.enabled);
    }

    #[test]
    fn test_scan_with_architecture() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(dir.path());

        let docs = dir.path().join("docs");
        let mut f = fs::File::create(docs.join("ARCHITECTURE.md")).unwrap();
        for i in 1..=5 {
            writeln!(f, "## {i}. Section {i}\n\nContent.\n").unwrap();
        }

        let config = scan_project(dir.path());
        assert!(config.architecture.enabled);
        assert_eq!(config.architecture.required_sections, 5);
    }

    #[test]
    fn test_scan_detects_type_pattern() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(dir.path());

        let ticket_dir = dir.path().join("docs/ticket");
        let mut f = fs::File::create(ticket_dir.join("PHIEU-1.md")).unwrap();
        writeln!(f, "# Phiếu 1\n\n**Loại:** `mutating`\n").unwrap();
        let mut f = fs::File::create(ticket_dir.join("PHIEU-2.md")).unwrap();
        writeln!(f, "# Phiếu 2\n\n**Loại:** `read-only`\n").unwrap();

        let config = scan_project(dir.path());
        assert!(!config.ticket.type_pattern.is_empty());
        assert!(config.ticket.type_pattern.contains("Loại"));
        assert!(config.ticket.valid_types.contains(&"mutating".to_string()));
        assert!(config.ticket.valid_types.contains(&"read-only".to_string()));
    }

    #[test]
    fn test_scan_detects_exclude_files() {
        let dir = tempfile::tempdir().unwrap();
        setup_project(dir.path());

        let ticket_dir = dir.path().join("docs/ticket");
        fs::File::create(ticket_dir.join("TICKET_TEMPLATE.md")).unwrap();
        fs::File::create(ticket_dir.join("README.md")).unwrap();
        let mut f = fs::File::create(ticket_dir.join("PHIEU-1.md")).unwrap();
        writeln!(f, "# Phiếu 1\n\n**Type:** `mutating`\n").unwrap();

        let config = scan_project(dir.path());
        assert!(
            config
                .ticket
                .exclude_files
                .contains(&"TICKET_TEMPLATE.md".to_string())
        );
        assert!(
            config
                .ticket
                .exclude_files
                .contains(&"README.md".to_string())
        );
    }

    #[test]
    fn test_scan_no_ticket_dir() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::File::create(docs.join("CHANGELOG.md")).unwrap();

        let config = scan_project(dir.path());
        assert!(config.ticket.type_pattern.is_empty());
        assert!(config.ticket.valid_types.is_empty());
    }

    #[test]
    fn test_write_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let content = write_config(dir.path(), &config).unwrap();
        assert!(content.contains("docs_dir"));
        assert!(dir.path().join(".docs-gate.toml").exists());
    }

    #[test]
    fn test_detect_type_pattern_english() {
        let contents = vec![
            "# T1\n\n**Type:** `mutating`\n".to_string(),
            "# T2\n\n**Type:** `read-only`\n".to_string(),
        ];
        let (pattern, types) = detect_type_pattern(&contents);
        assert!(!pattern.is_empty());
        assert!(pattern.contains("Type"));
        assert!(types.contains(&"mutating".to_string()));
    }

    #[test]
    fn test_detect_type_pattern_ignores_file_paths() {
        let contents = vec![
            "# T1\n\n**File cần sửa:** `src/app/api/route.ts`\n**Loại:** `mutating`\n".to_string(),
            "# T2\n\n**File cần sửa:** `src/lib/ai/digest.ts`\n**Loại:** `read-only`\n".to_string(),
        ];
        let (pattern, types) = detect_type_pattern(&contents);
        assert!(!pattern.is_empty());
        // Should pick Loại, not "File cần sửa" (which has path values)
        assert!(
            pattern.contains("Loại"),
            "Expected Loại pattern, got: {pattern}"
        );
        assert!(types.contains(&"mutating".to_string()));
    }

    #[test]
    fn test_detect_type_pattern_no_categorical_field() {
        // Only file path fields, no type field → should return None
        let contents = vec![
            "# T1\n\n**File:** `src/foo.ts`\n".to_string(),
            "# T2\n\n**File:** `src/bar.ts`\n".to_string(),
        ];
        let (pattern, _) = detect_type_pattern(&contents);
        assert!(pattern.is_empty());
    }

    #[test]
    fn test_detect_type_pattern_empty() {
        let (pattern, types) = detect_type_pattern(&[]);
        assert!(pattern.is_empty());
        assert!(types.is_empty());
    }
}
