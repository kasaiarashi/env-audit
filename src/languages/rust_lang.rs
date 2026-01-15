use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use crate::types::{EnvVarUsage, Language};
use super::LanguageScanner;

/// Scanner for Rust files
pub struct RustScanner;

static ENV_VAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:std::)?env::var\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap()
});

static ENV_VAR_OS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:std::)?env::var_os\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap()
});

static ENV_MACRO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"env!\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap()
});

static OPTION_ENV_MACRO: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"option_env!\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap()
});

impl RustScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for RustScanner {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let patterns: Vec<&Lazy<Regex>> = vec![
            &ENV_VAR,
            &ENV_VAR_OS,
            &ENV_MACRO,
            &OPTION_ENV_MACRO,
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
                                language: Language::Rust,
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
    fn test_env_var() {
        let scanner = RustScanner::new();
        let content = r#"let db = std::env::var("DATABASE_URL").unwrap();"#;
        let usages = scanner.scan(content, Path::new("main.rs"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }

    #[test]
    fn test_env_var_short() {
        let scanner = RustScanner::new();
        let content = r#"let port = env::var("PORT");"#;
        let usages = scanner.scan(content, Path::new("main.rs"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "PORT");
    }

    #[test]
    fn test_env_macro() {
        let scanner = RustScanner::new();
        let content = r#"const API_KEY: &str = env!("API_KEY");"#;
        let usages = scanner.scan(content, Path::new("main.rs"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "API_KEY");
    }

    #[test]
    fn test_option_env_macro() {
        let scanner = RustScanner::new();
        let content = r#"let debug = option_env!("DEBUG");"#;
        let usages = scanner.scan(content, Path::new("main.rs"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DEBUG");
    }
}
