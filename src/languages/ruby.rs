use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use crate::types::{EnvVarUsage, Language};
use super::LanguageScanner;

/// Scanner for Ruby files
pub struct RubyScanner;

static ENV_BRACKET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"ENV\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap()
});

static ENV_FETCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"ENV\.fetch\s*\(\s*['"]([A-Z_][A-Z0-9_]*)['"]"#).unwrap()
});

impl RubyScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RubyScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for RubyScanner {
    fn language(&self) -> Language {
        Language::Ruby
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["rb"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let patterns: Vec<&Lazy<Regex>> = vec![
            &ENV_BRACKET,
            &ENV_FETCH,
        ];

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
                            language: Language::Ruby,
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
    fn test_env_bracket() {
        let scanner = RubyScanner::new();
        let content = r#"db_url = ENV['DATABASE_URL']"#;
        let usages = scanner.scan(content, Path::new("config.rb"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }

    #[test]
    fn test_env_fetch() {
        let scanner = RubyScanner::new();
        let content = r#"port = ENV.fetch("PORT", "3000")"#;
        let usages = scanner.scan(content, Path::new("config.rb"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "PORT");
    }
}
