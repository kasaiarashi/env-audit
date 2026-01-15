use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::languages::LanguageRegistry;
use crate::types::EnvVarUsage;

/// Scans source code files for environment variable usage
pub struct CodeScanner {
    registry: LanguageRegistry,
}

impl CodeScanner {
    pub fn new() -> Self {
        Self {
            registry: LanguageRegistry::new(),
        }
    }

    /// Scan a single file for env var usages
    pub fn scan_file(&self, path: &Path) -> Result<Vec<EnvVarUsage>> {
        let content = std::fs::read_to_string(path)?;

        let scanner = match self.registry.get_scanner_for_file(path) {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        Ok(scanner.scan(&content, path))
    }

    /// Scan multiple files in parallel
    pub fn scan_files(&self, files: &[PathBuf]) -> Vec<EnvVarUsage> {
        files
            .par_iter()
            .filter_map(|path| {
                match self.scan_file(path) {
                    Ok(usages) => Some(usages),
                    Err(_) => None,
                }
            })
            .flatten()
            .collect()
    }
}

impl Default for CodeScanner {
    fn default() -> Self {
        Self::new()
    }
}
