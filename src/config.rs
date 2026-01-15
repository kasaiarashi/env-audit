use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

use crate::types::Severity;

/// Main configuration structure
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub scan: ScanConfig,

    #[serde(default)]
    pub naming: NamingConfig,

    #[serde(default)]
    pub output: OutputConfig,
}

/// Configuration for file scanning
#[derive(Debug, Deserialize)]
pub struct ScanConfig {
    /// Env files to parse (relative to project root)
    #[serde(default = "default_env_files")]
    pub env_files: Vec<String>,

    /// Glob patterns to include in scan
    #[serde(default = "default_include")]
    pub include: Vec<String>,

    /// Glob patterns to exclude from scan
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,

    /// Languages to scan (None = all supported languages)
    #[serde(default)]
    pub languages: Option<Vec<String>>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            env_files: default_env_files(),
            include: default_include(),
            exclude: default_exclude(),
            languages: None,
        }
    }
}

fn default_env_files() -> Vec<String> {
    vec![
        ".env".to_string(),
        ".env.local".to_string(),
        ".env.example".to_string(),
    ]
}

fn default_include() -> Vec<String> {
    vec![
        "**/*".to_string(),
    ]
}

fn default_exclude() -> Vec<String> {
    vec![
        "**/node_modules/**".to_string(),
        "**/target/**".to_string(),
        "**/vendor/**".to_string(),
        "**/.git/**".to_string(),
        "**/dist/**".to_string(),
        "**/build/**".to_string(),
        "**/__pycache__/**".to_string(),
        "**/venv/**".to_string(),
        "**/.venv/**".to_string(),
    ]
}

/// Configuration for naming convention checks
#[derive(Debug, Deserialize)]
pub struct NamingConfig {
    /// Use built-in naming conflict rules
    #[serde(default = "default_true")]
    pub builtin_rules: bool,

    /// Custom naming rules
    #[serde(default)]
    pub custom_rules: Vec<NamingRule>,

    /// Patterns to ignore (regex)
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            builtin_rules: true,
            custom_rules: Vec::new(),
            ignore_patterns: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

/// A custom naming rule
#[derive(Debug, Clone, Deserialize)]
pub struct NamingRule {
    /// Rule name for identification
    pub name: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,

    /// Alternative names that conflict with this rule
    #[serde(default)]
    pub alternatives: Vec<String>,

    /// The preferred name to use
    pub preferred: String,

    /// Severity level for this rule
    #[serde(default = "default_severity")]
    pub severity: String,
}

fn default_severity() -> String {
    "warning".to_string()
}

impl NamingRule {
    pub fn severity_level(&self) -> Severity {
        match self.severity.to_lowercase().as_str() {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            _ => Severity::Info,
        }
    }
}

/// Configuration for output formatting
#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    #[serde(default = "default_format")]
    pub format: String,

    /// Show suggestions for fixing issues
    #[serde(default = "default_true")]
    pub show_suggestions: bool,

    /// Group issues by type (true) or by file (false)
    #[serde(default = "default_true")]
    pub group_by_type: bool,

    /// Minimum severity to report
    #[serde(default = "default_min_severity")]
    pub min_severity: String,

    /// Output file path for non-terminal formats
    #[serde(default)]
    pub output_file: Option<String>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            show_suggestions: true,
            group_by_type: true,
            min_severity: default_min_severity(),
            output_file: None,
        }
    }
}

fn default_format() -> String {
    "terminal".to_string()
}

fn default_min_severity() -> String {
    "info".to_string()
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Generate a default configuration file content
    pub fn generate_default() -> String {
        r#"# env-audit configuration file

[scan]
# Env files to parse (relative to project root)
env_files = [".env", ".env.local", ".env.example"]

# Glob patterns to include in scan
include = ["**/*"]

# Glob patterns to exclude from scan
exclude = [
    "**/node_modules/**",
    "**/target/**",
    "**/vendor/**",
    "**/.git/**",
    "**/dist/**",
    "**/build/**",
]

# Languages to scan (comment out for all supported languages)
# languages = ["javascript", "typescript", "python", "rust", "go", "ruby", "php", "java", "csharp"]

[naming]
# Use built-in naming conflict rules
builtin_rules = true

# Patterns to ignore (regex) - vars matching these won't trigger naming issues
ignore_patterns = ["^_", "^INTERNAL_"]

# Custom naming rules
# [[naming.custom_rules]]
# name = "database-url"
# description = "Database connection URL naming"
# alternatives = ["DB_URL", "DB_CONNECTION"]
# preferred = "DATABASE_URL"
# severity = "warning"

[output]
# Default output format: "terminal", "json", "markdown", "html"
format = "terminal"

# Show suggestions for fixing issues
show_suggestions = true

# Group issues by type (true) or by file (false)
group_by_type = true

# Minimum severity to report: "error", "warning", "info"
min_severity = "info"

# Output file path for non-terminal formats (optional)
# output_file = "env-audit-report.json"
"#.to_string()
    }
}
