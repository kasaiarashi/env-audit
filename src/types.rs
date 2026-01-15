use serde::Serialize;
use std::path::PathBuf;

/// Supported programming languages for env var scanning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
    Rust,
    Go,
    Ruby,
    Php,
    Java,
    CSharp,
}

impl Language {
    /// Returns the file extensions associated with this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::JavaScript => &["js", "mjs", "cjs", "jsx"],
            Language::TypeScript => &["ts", "mts", "cts", "tsx"],
            Language::Python => &["py"],
            Language::Rust => &["rs"],
            Language::Go => &["go"],
            Language::Ruby => &["rb"],
            Language::Php => &["php"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
        }
    }

    /// Returns a human-readable name for the language
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Python => "Python",
            Language::Rust => "Rust",
            Language::Go => "Go",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Java => "Java",
            Language::CSharp => "C#",
        }
    }
}

/// Where an environment variable was found
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EnvVarSource {
    /// Defined in a .env file
    EnvFile { path: PathBuf, line: usize },
    /// Used in source code
    Code {
        path: PathBuf,
        line: usize,
        column: usize,
        language: Language,
    },
}

/// An environment variable definition (from .env files)
#[derive(Debug, Clone, Serialize)]
pub struct EnvVarDefinition {
    pub name: String,
    pub value: Option<String>,
    pub source_file: PathBuf,
    pub line: usize,
}

/// An environment variable usage (from source code)
#[derive(Debug, Clone, Serialize)]
pub struct EnvVarUsage {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub language: Language,
    /// The surrounding code context
    pub context: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Types of issues that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    /// Env var is used in code but not defined in any .env file
    MissingEnvVar,
    /// Env var is defined in .env but never used in code
    UnusedEnvVar,
    /// Env var name conflicts with naming conventions
    InconsistentNaming,
    /// Env var is defined multiple times
    DuplicateDefinition,
}

impl std::fmt::Display for IssueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueKind::MissingEnvVar => write!(f, "Missing env var"),
            IssueKind::UnusedEnvVar => write!(f, "Unused env var"),
            IssueKind::InconsistentNaming => write!(f, "Inconsistent naming"),
            IssueKind::DuplicateDefinition => write!(f, "Duplicate definition"),
        }
    }
}

/// A location in the codebase
#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file.display())?;
        if let Some(line) = self.line {
            write!(f, ":{}", line)?;
            if let Some(col) = self.column {
                write!(f, ":{}", col)?;
            }
        }
        Ok(())
    }
}

/// An issue found during the audit
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    pub kind: IssueKind,
    pub severity: Severity,
    pub var_name: String,
    pub message: String,
    pub locations: Vec<Location>,
    pub suggestion: Option<String>,
}

/// Summary statistics for a scan
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanSummary {
    pub files_scanned: usize,
    pub env_files_found: usize,
    pub vars_defined: usize,
    pub vars_used: usize,
    pub total_issues: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

/// The complete scan report
#[derive(Debug, Clone, Serialize)]
pub struct ScanReport {
    pub summary: ScanSummary,
    pub issues: Vec<Issue>,
    pub definitions: Vec<EnvVarDefinition>,
    pub usages: Vec<EnvVarUsage>,
    pub scan_duration_ms: u64,
}

impl ScanReport {
    pub fn new() -> Self {
        Self {
            summary: ScanSummary::default(),
            issues: Vec::new(),
            definitions: Vec::new(),
            usages: Vec::new(),
            scan_duration_ms: 0,
        }
    }

    /// Calculate summary statistics from the issues
    pub fn calculate_summary(&mut self) {
        self.summary.total_issues = self.issues.len();
        self.summary.errors = self.issues.iter().filter(|i| i.severity == Severity::Error).count();
        self.summary.warnings = self.issues.iter().filter(|i| i.severity == Severity::Warning).count();
        self.summary.infos = self.issues.iter().filter(|i| i.severity == Severity::Info).count();
        self.summary.vars_defined = self.definitions.len();
        self.summary.vars_used = self.usages.iter().map(|u| &u.name).collect::<std::collections::HashSet<_>>().len();
    }
}

impl Default for ScanReport {
    fn default() -> Self {
        Self::new()
    }
}
