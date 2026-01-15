use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use crate::config::ScanConfig;
use crate::types::Language;

/// Walks through project files respecting .gitignore and config exclusions
pub struct FileWalker {
    root: PathBuf,
    exclude_patterns: Vec<String>,
    languages: Option<Vec<Language>>,
}

impl FileWalker {
    pub fn new(root: &Path, config: &ScanConfig) -> Self {
        let languages = config.languages.as_ref().map(|langs| {
            langs.iter()
                .filter_map(|l| parse_language(l))
                .collect()
        });

        Self {
            root: root.to_path_buf(),
            exclude_patterns: config.exclude.clone(),
            languages,
        }
    }

    /// Find all source code files to scan
    pub fn find_source_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let walker = WalkBuilder::new(&self.root)
            .hidden(false)          // Don't skip hidden files by default
            .git_ignore(true)       // Respect .gitignore
            .git_global(true)       // Respect global gitignore
            .git_exclude(true)      // Respect .git/info/exclude
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check if path matches any exclude pattern
            if self.is_excluded(path) {
                continue;
            }

            // Check if file is a supported language
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if self.is_supported_extension(ext) {
                    files.push(path.to_path_buf());
                }
            }
        }

        Ok(files)
    }

    /// Find all .env files in the project
    pub fn find_env_files(&self, env_file_names: &[String]) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for name in env_file_names {
            let path = self.root.join(name);
            if path.exists() && path.is_file() {
                files.push(path);
            }
        }

        // Also search subdirectories for .env files
        let walker = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .max_depth(Some(3)) // Don't go too deep
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Check if it's a .env variant file
                if file_name.starts_with(".env") && !files.contains(&path.to_path_buf()) {
                    files.push(path.to_path_buf());
                }
            }
        }

        Ok(files)
    }

    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.exclude_patterns {
            // Simple glob matching for common patterns
            if pattern.contains("**") {
                // Handle **/dir/** pattern
                let pattern_parts: Vec<&str> = pattern.split("**").collect();
                if pattern_parts.len() == 2 {
                    let middle = pattern_parts[1].trim_matches('/');
                    if path_str.contains(&format!("/{}/", middle)) ||
                       path_str.contains(&format!("\\{}\\", middle)) {
                        return true;
                    }
                }
            } else if path_str.contains(pattern.trim_matches('*')) {
                return true;
            }
        }

        false
    }

    fn is_supported_extension(&self, ext: &str) -> bool {
        let ext_lower = ext.to_lowercase();

        // Get list of allowed languages
        let languages = match &self.languages {
            Some(langs) => langs.clone(),
            None => all_languages(),
        };

        // Check if extension matches any allowed language
        for lang in languages {
            if lang.extensions().contains(&ext_lower.as_str()) {
                return true;
            }
        }

        false
    }
}

/// Get the language for a file based on its extension
#[allow(dead_code)]
pub fn get_language_for_file(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    for lang in all_languages() {
        if lang.extensions().contains(&ext.as_str()) {
            return Some(lang);
        }
    }

    None
}

fn all_languages() -> Vec<Language> {
    vec![
        Language::JavaScript,
        Language::TypeScript,
        Language::Python,
        Language::Rust,
        Language::Go,
        Language::Ruby,
        Language::Php,
        Language::Java,
        Language::CSharp,
    ]
}

fn parse_language(name: &str) -> Option<Language> {
    match name.to_lowercase().as_str() {
        "javascript" | "js" => Some(Language::JavaScript),
        "typescript" | "ts" => Some(Language::TypeScript),
        "python" | "py" => Some(Language::Python),
        "rust" | "rs" => Some(Language::Rust),
        "go" => Some(Language::Go),
        "ruby" | "rb" => Some(Language::Ruby),
        "php" => Some(Language::Php),
        "java" => Some(Language::Java),
        "csharp" | "cs" | "c#" => Some(Language::CSharp),
        _ => None,
    }
}
