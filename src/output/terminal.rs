use anyhow::Result;
use colored::Colorize;
use comfy_table::{Cell, Color, ContentArrangement, Table};

use crate::types::{IssueKind, ScanReport, Severity};
use super::OutputFormatter;

pub struct TerminalOutput {
    pub no_color: bool,
    pub show_suggestions: bool,
}

impl TerminalOutput {
    pub fn new(no_color: bool, show_suggestions: bool) -> Self {
        if no_color {
            colored::control::set_override(false);
        }
        Self { no_color, show_suggestions }
    }

    fn severity_color(&self, severity: Severity) -> Color {
        match severity {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
            Severity::Info => Color::Cyan,
        }
    }

    fn severity_symbol(&self, severity: Severity) -> &'static str {
        match severity {
            Severity::Error => "x",
            Severity::Warning => "!",
            Severity::Info => "i",
        }
    }
}

impl OutputFormatter for TerminalOutput {
    fn format(&self, report: &ScanReport) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!("\n{}\n\n", "env-audit scan results".bold()));

        // Summary stats
        output.push_str(&format!(
            "Files scanned: {}  |  Env files: {}  |  Duration: {}ms\n",
            report.summary.files_scanned,
            report.summary.env_files_found,
            report.scan_duration_ms
        ));
        output.push_str(&format!(
            "Vars defined: {}  |  Vars used: {}\n\n",
            report.summary.vars_defined,
            report.summary.vars_used
        ));

        if report.issues.is_empty() {
            output.push_str(&format!("{}\n", "No issues found!".green().bold()));
            return Ok(output);
        }

        // Group issues by kind
        let missing: Vec<_> = report.issues.iter()
            .filter(|i| i.kind == IssueKind::MissingEnvVar)
            .collect();
        let unused: Vec<_> = report.issues.iter()
            .filter(|i| i.kind == IssueKind::UnusedEnvVar)
            .collect();
        let naming: Vec<_> = report.issues.iter()
            .filter(|i| i.kind == IssueKind::InconsistentNaming)
            .collect();

        // Missing env vars
        if !missing.is_empty() {
            output.push_str(&format!("{} ({})\n", "MISSING ENV VARS".red().bold(), missing.len()));

            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec![
                Cell::new("").fg(Color::White),
                Cell::new("Variable").fg(Color::White),
                Cell::new("Used In").fg(Color::White),
            ]);

            for issue in &missing {
                let locations: Vec<String> = issue.locations.iter()
                    .take(3)
                    .map(|l| l.to_string())
                    .collect();
                let location_str = if issue.locations.len() > 3 {
                    format!("{} (+{} more)", locations.join("\n"), issue.locations.len() - 3)
                } else {
                    locations.join("\n")
                };

                table.add_row(vec![
                    Cell::new(self.severity_symbol(issue.severity))
                        .fg(self.severity_color(issue.severity)),
                    Cell::new(&issue.var_name),
                    Cell::new(location_str),
                ]);
            }
            output.push_str(&format!("{}\n\n", table));
        }

        // Unused env vars
        if !unused.is_empty() {
            output.push_str(&format!("{} ({})\n", "UNUSED ENV VARS".yellow().bold(), unused.len()));

            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec![
                Cell::new("").fg(Color::White),
                Cell::new("Variable").fg(Color::White),
                Cell::new("Defined In").fg(Color::White),
            ]);

            for issue in &unused {
                let locations: Vec<String> = issue.locations.iter()
                    .map(|l| l.to_string())
                    .collect();

                table.add_row(vec![
                    Cell::new(self.severity_symbol(issue.severity))
                        .fg(self.severity_color(issue.severity)),
                    Cell::new(&issue.var_name),
                    Cell::new(locations.join("\n")),
                ]);
            }
            output.push_str(&format!("{}\n\n", table));
        }

        // Naming convention issues
        if !naming.is_empty() {
            output.push_str(&format!("{} ({})\n", "NAMING ISSUES".cyan().bold(), naming.len()));

            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec![
                Cell::new("").fg(Color::White),
                Cell::new("Variable").fg(Color::White),
                Cell::new("Suggestion").fg(Color::White),
            ]);

            for issue in &naming {
                table.add_row(vec![
                    Cell::new(self.severity_symbol(issue.severity))
                        .fg(self.severity_color(issue.severity)),
                    Cell::new(&issue.var_name),
                    Cell::new(issue.suggestion.as_deref().unwrap_or("")),
                ]);
            }
            output.push_str(&format!("{}\n\n", table));
        }

        // Summary
        output.push_str(&format!("{}\n", "SUMMARY".bold()));

        let errors_str = format!("Errors: {}", report.summary.errors);
        let warnings_str = format!("Warnings: {}", report.summary.warnings);
        let info_str = format!("Info: {}", report.summary.infos);

        output.push_str(&format!(
            "  {}  |  {}  |  {}\n",
            if report.summary.errors > 0 { errors_str.red().to_string() } else { errors_str },
            if report.summary.warnings > 0 { warnings_str.yellow().to_string() } else { warnings_str },
            info_str.cyan()
        ));

        Ok(output)
    }
}
