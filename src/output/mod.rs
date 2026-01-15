mod terminal;
mod json;
mod markdown;
mod html;

pub use terminal::TerminalOutput;
pub use json::JsonOutput;
pub use markdown::MarkdownOutput;
pub use html::HtmlOutput;

use std::path::Path;
use anyhow::Result;

use crate::types::ScanReport;

/// Trait for output formatters
pub trait OutputFormatter {
    /// Format the report and write to the given writer
    fn format(&self, report: &ScanReport) -> Result<String>;

    /// Write the formatted output to a file
    fn write_to_file(&self, report: &ScanReport, path: &Path) -> Result<()> {
        let output = self.format(report)?;
        std::fs::write(path, output)?;
        Ok(())
    }
}
