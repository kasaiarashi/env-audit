use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use super::LanguageScanner;
use crate::types::{EnvVarUsage, Language};

/// Scanner for Go files
pub struct GoScanner;

static OS_GETENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"os\.Getenv\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap());

static OS_LOOKUP_ENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"os\.LookupEnv\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap());

static OS_SETENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"os\.Setenv\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap());

impl GoScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for GoScanner {
    fn language(&self) -> Language {
        Language::Go
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["go"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let patterns: Vec<&Lazy<Regex>> = vec![&OS_GETENV, &OS_LOOKUP_ENV, &OS_SETENV];

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
                            language: Language::Go,
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
    fn test_os_getenv() {
        let scanner = GoScanner::new();
        let content = r#"port := os.Getenv("PORT")"#;
        let usages = scanner.scan(content, Path::new("main.go"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "PORT");
    }

    #[test]
    fn test_os_lookup_env() {
        let scanner = GoScanner::new();
        let content = r#"if val, ok := os.LookupEnv("DATABASE_URL"); ok {"#;
        let usages = scanner.scan(content, Path::new("main.go"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }
}
