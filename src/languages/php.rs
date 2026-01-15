use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use super::LanguageScanner;
use crate::types::{EnvVarUsage, Language};

/// Scanner for PHP files
pub struct PhpScanner;

static GETENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"getenv\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap());

static DOLLAR_ENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\$_ENV\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap());

static DOLLAR_SERVER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\$_SERVER\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap());

// Laravel env() helper
static LARAVEL_ENV: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"\benv\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap());

impl PhpScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PhpScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for PhpScanner {
    fn language(&self) -> Language {
        Language::Php
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["php"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let patterns: Vec<&Lazy<Regex>> = vec![&GETENV, &DOLLAR_ENV, &DOLLAR_SERVER, &LARAVEL_ENV];

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
                            language: Language::Php,
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
    fn test_getenv() {
        let scanner = PhpScanner::new();
        let content = r#"$dbUrl = getenv('DATABASE_URL');"#;
        let usages = scanner.scan(content, Path::new("config.php"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }

    #[test]
    fn test_dollar_env() {
        let scanner = PhpScanner::new();
        let content = r#"$apiKey = $_ENV['API_KEY'];"#;
        let usages = scanner.scan(content, Path::new("config.php"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "API_KEY");
    }

    #[test]
    fn test_laravel_env() {
        let scanner = PhpScanner::new();
        let content = r#"'debug' => env('APP_DEBUG', false),"#;
        let usages = scanner.scan(content, Path::new("config/app.php"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "APP_DEBUG");
    }
}
