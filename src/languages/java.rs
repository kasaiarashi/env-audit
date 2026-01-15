use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use super::LanguageScanner;
use crate::types::{EnvVarUsage, Language};

/// Scanner for Java files
pub struct JavaScanner;

static SYSTEM_GETENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"System\.getenv\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap());

static SYSTEM_GETPROPERTY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"System\.getProperty\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap());

impl JavaScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for JavaScanner {
    fn language(&self) -> Language {
        Language::Java
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["java"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let patterns: Vec<&Lazy<Regex>> = vec![&SYSTEM_GETENV, &SYSTEM_GETPROPERTY];

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            for pattern in &patterns {
                for cap in pattern.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        usages.push(EnvVarUsage {
                            name: m.as_str().to_string(),
                            file_path: file_path.to_path_buf(),
                            line: line_num,
                            column: m.start() + 1,
                            language: Language::Java,
                            context: Some(line.trim().to_string()),
                        });
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
    fn test_system_getenv() {
        let scanner = JavaScanner::new();
        let content = r#"String dbUrl = System.getenv("DATABASE_URL");"#;
        let usages = scanner.scan(content, Path::new("Config.java"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }

    #[test]
    fn test_system_getproperty() {
        let scanner = JavaScanner::new();
        let content = r#"String port = System.getProperty("PORT");"#;
        let usages = scanner.scan(content, Path::new("Config.java"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "PORT");
    }
}
