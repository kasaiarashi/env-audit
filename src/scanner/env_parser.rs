use anyhow::{Context, Result};
use std::path::Path;

use crate::types::EnvVarDefinition;

/// Parse a .env file and extract all variable definitions
pub fn parse_env_file(path: &Path) -> Result<Vec<EnvVarDefinition>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read env file: {}", path.display()))?;

    let mut definitions = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num + 1; // 1-indexed
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse KEY=value format
        if let Some((name, value)) = parse_env_line(line) {
            definitions.push(EnvVarDefinition {
                name,
                value: Some(value),
                source_file: path.to_path_buf(),
                line: line_num,
            });
        }
    }

    Ok(definitions)
}

/// Parse a single line from an env file
/// Returns (key, value) if valid, None otherwise
fn parse_env_line(line: &str) -> Option<(String, String)> {
    // Handle export prefix
    let line = line.strip_prefix("export ").unwrap_or(line);

    // Find the first '=' separator
    let eq_pos = line.find('=')?;

    let key = line[..eq_pos].trim();
    let value = line[eq_pos + 1..].trim();

    // Validate key - must be a valid env var name
    if !is_valid_env_var_name(key) {
        return None;
    }

    // Remove surrounding quotes from value
    let value = strip_quotes(value);

    Some((key.to_string(), value.to_string()))
}

/// Check if a string is a valid environment variable name
fn is_valid_env_var_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be a letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Remove surrounding quotes from a string
fn strip_quotes(s: &str) -> String {
    let s = s.trim();

    // Check for matching quotes
    if (s.starts_with('"') && s.ends_with('"')) ||
       (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 {
            return s[1..s.len()-1].to_string();
        }
    }

    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_line_simple() {
        let (key, value) = parse_env_line("DATABASE_URL=postgres://localhost/db").unwrap();
        assert_eq!(key, "DATABASE_URL");
        assert_eq!(value, "postgres://localhost/db");
    }

    #[test]
    fn test_parse_env_line_with_quotes() {
        let (key, value) = parse_env_line("SECRET_KEY=\"my secret value\"").unwrap();
        assert_eq!(key, "SECRET_KEY");
        assert_eq!(value, "my secret value");
    }

    #[test]
    fn test_parse_env_line_with_export() {
        let (key, value) = parse_env_line("export API_KEY=abc123").unwrap();
        assert_eq!(key, "API_KEY");
        assert_eq!(value, "abc123");
    }

    #[test]
    fn test_parse_env_line_empty_value() {
        let (key, value) = parse_env_line("EMPTY_VAR=").unwrap();
        assert_eq!(key, "EMPTY_VAR");
        assert_eq!(value, "");
    }

    #[test]
    fn test_is_valid_env_var_name() {
        assert!(is_valid_env_var_name("DATABASE_URL"));
        assert!(is_valid_env_var_name("_PRIVATE"));
        assert!(is_valid_env_var_name("API_KEY_123"));
        assert!(!is_valid_env_var_name("123_INVALID"));
        assert!(!is_valid_env_var_name(""));
        assert!(!is_valid_env_var_name("has-dash"));
    }
}
