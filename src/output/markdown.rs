use anyhow::Result;

use super::OutputFormatter;
use crate::types::{IssueKind, ScanReport, Severity};

pub struct MarkdownOutput;

impl MarkdownOutput {
    pub fn new() -> Self {
        Self
    }

    fn severity_emoji(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => ":x:",
            Severity::Warning => ":warning:",
            Severity::Info => ":information_source:",
        }
    }
}

impl Default for MarkdownOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for MarkdownOutput {
    fn format(&self, report: &ScanReport) -> Result<String> {
        let mut output = String::new();

        // Header
        output.push_str("# env-audit Report\n\n");

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- **Files scanned:** {}\n",
            report.summary.files_scanned
        ));
        output.push_str(&format!(
            "- **Env files found:** {}\n",
            report.summary.env_files_found
        ));
        output.push_str(&format!(
            "- **Variables defined:** {}\n",
            report.summary.vars_defined
        ));
        output.push_str(&format!(
            "- **Variables used:** {}\n",
            report.summary.vars_used
        ));
        output.push_str(&format!(
            "- **Scan duration:** {}ms\n\n",
            report.scan_duration_ms
        ));

        // Issues summary
        output.push_str("### Issues\n\n");
        output.push_str("| Severity | Count |\n");
        output.push_str("|----------|-------|\n");
        output.push_str(&format!("| :x: Errors | {} |\n", report.summary.errors));
        output.push_str(&format!(
            "| :warning: Warnings | {} |\n",
            report.summary.warnings
        ));
        output.push_str(&format!(
            "| :information_source: Info | {} |\n\n",
            report.summary.infos
        ));

        if report.issues.is_empty() {
            output.push_str("> **No issues found!** :tada:\n");
            return Ok(output);
        }

        // Group issues by kind
        let missing: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.kind == IssueKind::MissingEnvVar)
            .collect();
        let unused: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.kind == IssueKind::UnusedEnvVar)
            .collect();
        let naming: Vec<_> = report
            .issues
            .iter()
            .filter(|i| i.kind == IssueKind::InconsistentNaming)
            .collect();

        // Missing env vars
        if !missing.is_empty() {
            output.push_str("## Missing Environment Variables\n\n");
            output.push_str(
                "These variables are used in code but not defined in any `.env` file.\n\n",
            );
            output.push_str("| | Variable | Used In |\n");
            output.push_str("|---|----------|----------|\n");

            for issue in &missing {
                let locations: Vec<String> = issue
                    .locations
                    .iter()
                    .take(3)
                    .map(|l| format!("`{}`", l))
                    .collect();
                let location_str = if issue.locations.len() > 3 {
                    format!(
                        "{} (+{} more)",
                        locations.join(", "),
                        issue.locations.len() - 3
                    )
                } else {
                    locations.join(", ")
                };

                output.push_str(&format!(
                    "| {} | `{}` | {} |\n",
                    Self::severity_emoji(issue.severity),
                    issue.var_name,
                    location_str
                ));
            }
            output.push('\n');
        }

        // Unused env vars
        if !unused.is_empty() {
            output.push_str("## Unused Environment Variables\n\n");
            output.push_str(
                "These variables are defined in `.env` files but never used in code.\n\n",
            );
            output.push_str("| | Variable | Defined In |\n");
            output.push_str("|---|----------|------------|\n");

            for issue in &unused {
                let locations: Vec<String> =
                    issue.locations.iter().map(|l| format!("`{}`", l)).collect();

                output.push_str(&format!(
                    "| {} | `{}` | {} |\n",
                    Self::severity_emoji(issue.severity),
                    issue.var_name,
                    locations.join(", ")
                ));
            }
            output.push('\n');
        }

        // Naming convention issues
        if !naming.is_empty() {
            output.push_str("## Naming Convention Issues\n\n");
            output.push_str("These variables could be renamed for better consistency.\n\n");
            output.push_str("| | Variable | Suggestion |\n");
            output.push_str("|---|----------|------------|\n");

            for issue in &naming {
                output.push_str(&format!(
                    "| {} | `{}` | {} |\n",
                    Self::severity_emoji(issue.severity),
                    issue.var_name,
                    issue.suggestion.as_deref().unwrap_or("")
                ));
            }
            output.push('\n');
        }

        output.push_str("---\n\n");
        output.push_str("*Generated by [env-audit](https://github.com/example/env-audit)*\n");

        Ok(output)
    }
}
