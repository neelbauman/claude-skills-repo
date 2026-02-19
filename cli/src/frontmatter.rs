use std::collections::HashMap;

/// Parse YAML frontmatter from a markdown file content.
/// Supports both `>` (folded) and `|` (literal) multiline values.
pub fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    // Extract content between --- delimiters
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return map;
    }

    let after_first = &trimmed[3..];
    let end = match after_first.find("\n---") {
        Some(pos) => pos,
        None => return map,
    };

    let fm_block = &after_first[..end];
    let mut current_key: Option<String> = None;
    let mut is_multiline = false;

    for line in fm_block.lines() {
        // Continuation of multiline value (indented with spaces)
        if is_multiline && current_key.is_some() && (line.starts_with("  ") || line.starts_with('\t'))
        {
            let key = current_key.as_ref().unwrap();
            let existing = map.get(key).cloned().unwrap_or_default();
            let appended = if existing.is_empty() {
                line.trim().to_string()
            } else {
                format!("{} {}", existing, line.trim())
            };
            map.insert(key.clone(), appended);
            continue;
        }

        // Try to match key: value
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim();
            if key.is_empty() || key.contains(' ') {
                current_key = None;
                is_multiline = false;
                continue;
            }
            let val = line[colon_pos + 1..].trim();
            if val == ">" || val == "|" {
                current_key = Some(key.to_string());
                is_multiline = true;
                map.insert(key.to_string(), String::new());
            } else {
                current_key = Some(key.to_string());
                is_multiline = false;
                map.insert(key.to_string(), val.to_string());
            }
        } else {
            current_key = None;
            is_multiline = false;
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folded_scalar() {
        let content = "---\nname: test\ndescription: >\n  line one\n  line two\ntags: a, b\n---\n# Body";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("name").unwrap(), "test");
        assert_eq!(fm.get("description").unwrap(), "line one line two");
        assert_eq!(fm.get("tags").unwrap(), "a, b");
    }

    #[test]
    fn test_literal_scalar() {
        let content = "---\nname: agent\ndescription: |\n  first\n  second\ntools: Read, Grep\n---\n";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("description").unwrap(), "first second");
        assert_eq!(fm.get("tools").unwrap(), "Read, Grep");
    }

    #[test]
    fn test_no_frontmatter() {
        let fm = parse_frontmatter("# Just a heading");
        assert!(fm.is_empty());
    }
}
