use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "env-audit")]
#[command(
    author,
    version,
    about = "Scan projects for environment variable issues"
)]
#[command(long_about = "env-audit scans your project for:\n\n\
  - Missing env vars: used in code but not defined in .env files\n\
  - Unused env vars: defined in .env but never used in code\n\
  - Inconsistent naming: conflicts like DB_URL vs DATABASE_URL")]
pub struct Cli {
    /// Path to config file
    #[arg(short, long, default_value = ".env-audit.toml")]
    pub config: PathBuf,

    /// Project path to scan
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Terminal)]
    pub format: OutputFormat,

    /// Output file path (for json/markdown/html formats)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan project for environment variable issues (default)
    Scan(ScanArgs),

    /// Run as CI check (exits with error code if issues found)
    Check(CheckArgs),

    /// Generate a default .env-audit.toml config file
    Init,

    /// List all detected environment variables
    List(ListArgs),

    /// Compare two env files
    Compare(CompareArgs),
}

#[derive(Parser, Default)]
pub struct ScanArgs {
    /// Only check for missing env vars
    #[arg(long)]
    pub missing: bool,

    /// Only check for unused env vars
    #[arg(long)]
    pub unused: bool,

    /// Only check naming conventions
    #[arg(long)]
    pub naming: bool,

    /// Additional env files to check
    #[arg(long = "env-file", value_name = "FILE")]
    pub env_files: Vec<PathBuf>,

    /// Additional patterns to ignore
    #[arg(long, value_name = "PATTERN")]
    pub ignore: Vec<String>,

    /// Filter by language (can be repeated)
    #[arg(long, value_name = "LANG")]
    pub language: Vec<String>,

    /// Minimum severity level to report
    #[arg(long, value_enum, default_value_t = SeverityFilter::Info)]
    pub severity: SeverityFilter,
}

#[derive(Parser)]
pub struct CheckArgs {
    /// Fail if issues of this severity or higher are found
    #[arg(long, value_enum, default_value_t = SeverityFilter::Error)]
    pub fail_on: SeverityFilter,

    /// Print summary only, not individual issues
    #[arg(long)]
    pub summary: bool,
}

#[derive(Parser)]
pub struct ListArgs {
    /// Show only defined vars
    #[arg(long)]
    pub defined: bool,

    /// Show only used vars
    #[arg(long)]
    pub used: bool,

    /// Include file locations
    #[arg(long)]
    pub locations: bool,
}

#[derive(Parser)]
pub struct CompareArgs {
    /// First env file
    pub file1: PathBuf,

    /// Second env file
    pub file2: PathBuf,

    /// Show actual values (warning: may expose secrets)
    #[arg(long)]
    pub show_values: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Terminal,
    Json,
    Markdown,
    Html,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SeverityFilter {
    #[default]
    Info,
    Warning,
    Error,
}

impl From<SeverityFilter> for crate::types::Severity {
    fn from(filter: SeverityFilter) -> Self {
        match filter {
            SeverityFilter::Info => crate::types::Severity::Info,
            SeverityFilter::Warning => crate::types::Severity::Warning,
            SeverityFilter::Error => crate::types::Severity::Error,
        }
    }
}

impl ScanArgs {
    /// Returns true if no specific check type is selected (meaning run all)
    pub fn run_all_checks(&self) -> bool {
        !self.missing && !self.unused && !self.naming
    }
}
