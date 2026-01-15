mod csharp;
mod go;
mod java;
mod javascript;
mod php;
mod python;
mod ruby;
mod rust_lang;

use std::path::Path;

use crate::types::{EnvVarUsage, Language};

/// Trait for language-specific env var scanning
pub trait LanguageScanner: Send + Sync {
    /// Returns the language this scanner handles
    fn language(&self) -> Language;

    /// Returns file extensions this scanner handles
    fn extensions(&self) -> &'static [&'static str];

    /// Scan content for env var usages
    fn scan(&self, content: &str, file_path: &Path) -> Vec<EnvVarUsage>;
}

/// Registry of all language scanners
pub struct LanguageRegistry {
    scanners: Vec<Box<dyn LanguageScanner>>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        Self {
            scanners: vec![
                Box::new(javascript::JavaScriptScanner::new()),
                Box::new(python::PythonScanner::new()),
                Box::new(rust_lang::RustScanner::new()),
                Box::new(go::GoScanner::new()),
                Box::new(ruby::RubyScanner::new()),
                Box::new(php::PhpScanner::new()),
                Box::new(java::JavaScanner::new()),
                Box::new(csharp::CSharpScanner::new()),
            ],
        }
    }

    /// Get the appropriate scanner for a file based on its extension
    pub fn get_scanner_for_file(&self, path: &Path) -> Option<&dyn LanguageScanner> {
        let ext = path.extension()?.to_str()?.to_lowercase();

        for scanner in &self.scanners {
            if scanner.extensions().contains(&ext.as_str()) {
                return Some(scanner.as_ref());
            }
        }

        None
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
