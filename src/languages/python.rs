use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use super::LanguageScanner;
use crate::types::{EnvVarUsage, Language};

/// Scanner for Python files
pub struct PythonScanner;

// Patterns for detecting env var usage in Python
static OS_ENVIRON_BRACKET: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"os\.environ\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap());

static OS_ENVIRON_GET: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"os\.environ\.get\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap());

static OS_GETENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"os\.getenv\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap());

// When `from os import environ` is used
static ENVIRON_BRACKET: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\benviron\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap());

static ENVIRON_GET: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\benviron\.get\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap());

// When `from os import getenv` is used
static GETENV_DIRECT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\bgetenv\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap());

impl PythonScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for PythonScanner {
    fn language(&self) -> Language {
        Language::Python
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["py"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let patterns: Vec<&Lazy<Regex>> = vec![
            &OS_ENVIRON_BRACKET,
            &OS_ENVIRON_GET,
            &OS_GETENV,
            &ENVIRON_BRACKET,
            &ENVIRON_GET,
            &GETENV_DIRECT,
        ];

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            for pattern in &patterns {
                for cap in pattern.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        let key = (line_num, m.start(), m.as_str().to_string());
                        if seen.insert(key) {
                            usages.push(EnvVarUsage {
                                name: m.as_str().to_string(),
                                file_path: file_path.to_path_buf(),
                                line: line_num,
                                column: m.start() + 1,
                                language: Language::Python,
                                context: Some(line.trim().to_string()),
                            });
                        }
                    }
                }
            }
        }

        usages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_environ_bracket() {
        let scanner = PythonScanner::new();
        let content = r#"db_url = os.environ['DATABASE_URL']"#;
        let usages = scanner.scan(content, Path::new("test.py"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }

    #[test]
    fn test_os_environ_get() {
        let scanner = PythonScanner::new();
        let content = r#"api_key = os.environ.get('API_KEY', 'default')"#;
        let usages = scanner.scan(content, Path::new("test.py"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "API_KEY");
    }

    #[test]
    fn test_os_getenv() {
        let scanner = PythonScanner::new();
        let content = r#"port = os.getenv("PORT")"#;
        let usages = scanner.scan(content, Path::new("test.py"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "PORT");
    }

    #[test]
    fn test_environ_direct() {
        let scanner = PythonScanner::new();
        let content = r#"
from os import environ
secret = environ['SECRET_KEY']
"#;
        let usages = scanner.scan(content, Path::new("test.py"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "SECRET_KEY");
    }

    #[test]
    fn test_getenv_direct() {
        let scanner = PythonScanner::new();
        let content = r#"
from os import getenv
debug = getenv('DEBUG')
"#;
        let usages = scanner.scan(content, Path::new("test.py"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DEBUG");
    }
}
