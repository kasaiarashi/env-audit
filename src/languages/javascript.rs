use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

use crate::types::{EnvVarUsage, Language};
use super::LanguageScanner;

/// Scanner for JavaScript and TypeScript files
pub struct JavaScriptScanner;

// Patterns for detecting env var usage in JS/TS
static PROCESS_ENV_DOT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"process\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap()
});

static PROCESS_ENV_BRACKET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"process\.env\[['"]([A-Z_][A-Z0-9_]*)['"]\]"#).unwrap()
});

static IMPORT_META_ENV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\.meta\.env\.([A-Z_][A-Z0-9_]*)"#).unwrap()
});

static DESTRUCTURE_PROCESS_ENV: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:const|let|var)\s*\{\s*([^}]+)\s*\}\s*=\s*process\.env"#).unwrap()
});

impl JavaScriptScanner {
    pub fn new() -> Self {
        Self
    }

    fn extract_destructured_vars(capture: &str) -> Vec<String> {
        capture
            .split(',')
            .filter_map(|s| {
                let s = s.trim();
                // Handle renaming: VAR_NAME: localName
                let name = s.split(':').next()?.trim();
                // Only valid env var names (uppercase with underscores)
                if name.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                    && !name.is_empty()
                    && name.chars().next().map(|c| c.is_ascii_uppercase() || c == '_').unwrap_or(false)
                {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for JavaScriptScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageScanner for JavaScriptScanner {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"]
    }

    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage> {
        let mut usages = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1;

            // process.env.VAR_NAME
            for cap in PROCESS_ENV_DOT.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    usages.push(EnvVarUsage {
                        name: m.as_str().to_string(),
                        file_path: file_path.to_path_buf(),
                        line: line_num,
                        column: m.start() + 1,
                        language: Language::JavaScript,
                        context: Some(line.trim().to_string()),
                    });
                }
            }

            // process.env['VAR_NAME'] or process.env["VAR_NAME"]
            for cap in PROCESS_ENV_BRACKET.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    usages.push(EnvVarUsage {
                        name: m.as_str().to_string(),
                        file_path: file_path.to_path_buf(),
                        line: line_num,
                        column: m.start() + 1,
                        language: Language::JavaScript,
                        context: Some(line.trim().to_string()),
                    });
                }
            }

            // import.meta.env.VAR_NAME (Vite)
            for cap in IMPORT_META_ENV.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    usages.push(EnvVarUsage {
                        name: m.as_str().to_string(),
                        file_path: file_path.to_path_buf(),
                        line: line_num,
                        column: m.start() + 1,
                        language: Language::JavaScript,
                        context: Some(line.trim().to_string()),
                    });
                }
            }

            // const { VAR1, VAR2 } = process.env
            for cap in DESTRUCTURE_PROCESS_ENV.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    for var_name in Self::extract_destructured_vars(m.as_str()) {
                        usages.push(EnvVarUsage {
                            name: var_name,
                            file_path: file_path.to_path_buf(),
                            line: line_num,
                            column: m.start() + 1,
                            language: Language::JavaScript,
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
    fn test_process_env_dot() {
        let scanner = JavaScriptScanner::new();
        let content = r#"const apiKey = process.env.API_KEY;"#;
        let usages = scanner.scan(content, Path::new("test.js"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "API_KEY");
    }

    #[test]
    fn test_process_env_bracket() {
        let scanner = JavaScriptScanner::new();
        let content = r#"const key = process.env['SECRET_KEY'];"#;
        let usages = scanner.scan(content, Path::new("test.js"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "SECRET_KEY");
    }

    #[test]
    fn test_import_meta_env() {
        let scanner = JavaScriptScanner::new();
        let content = r#"const url = import.meta.env.VITE_API_URL;"#;
        let usages = scanner.scan(content, Path::new("test.ts"));
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].name, "VITE_API_URL");
    }

    #[test]
    fn test_destructuring() {
        let scanner = JavaScriptScanner::new();
        let content = r#"const { PORT, HOST, DATABASE_URL } = process.env;"#;
        let usages = scanner.scan(content, Path::new("test.js"));
        assert_eq!(usages.len(), 3);
        let names: Vec<_> = usages.iter().map(|u| u.name.as_str()).collect();
        assert!(names.contains(&"PORT"));
        assert!(names.contains(&"HOST"));
        assert!(names.contains(&"DATABASE_URL"));
    }

    #[test]
    fn test_multiple_usages() {
        let scanner = JavaScriptScanner::new();
        let content = r#"
            const apiKey = process.env.API_KEY;
            const dbUrl = process.env.DATABASE_URL;
            const port = process.env["PORT"];
        "#;
        let usages = scanner.scan(content, Path::new("test.js"));
        assert_eq!(usages.len(), 3);
    }
}
