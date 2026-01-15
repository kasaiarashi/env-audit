use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use crate::types::{EnvVarUsage, Language};
use super::LanguageScanner;

/// Scanner for C# files
pub struct CSharpScanner;

static ENVIRONMENT_GETENV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"Environment\.GetEnvironmentVariable\s*\(\s*"([A-Z_][A-Z0-9_]*)""#).unwrap()
});

static CONFIG_MANAGER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"ConfigurationManager\.AppSettings\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap()
});

impl CSharpScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CSharpScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for CSharpScanner {
    fn language(&self) -> Language {
        Language::CSharp
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["cs"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();
        let patterns: Vec<&Lazy<Regex>> = vec![
            &ENVIRONMENT_GETENV,
            &CONFIG_MANAGER,
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
                            language: Language::CSharp,
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
    fn test_environment_getenv() {
        let scanner = CSharpScanner::new();
        let content = r#"var dbUrl = Environment.GetEnvironmentVariable("DATABASE_URL");"#;
        let usages = scanner.scan(content, Path::new("Config.cs"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "DATABASE_URL");
    }

    #[test]
    fn test_config_manager() {
        let scanner = CSharpScanner::new();
        let content = r#"var apiKey = ConfigurationManager.AppSettings["API_KEY"];"#;
        let usages = scanner.scan(content, Path::new("Config.cs"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "API_KEY");
    }
}
