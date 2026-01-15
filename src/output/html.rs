use anyhow::Result;

use crate::types::{IssueKind, ScanReport, Severity};
use super::OutputFormatter;

pub struct HtmlOutput;

impl HtmlOutput {
    pub fn new() -> Self {
        Self
    }

    fn severity_class(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }

    fn severity_label(severity: Severity) -> &'static str {
        match severity {
            Severity::Error => "Error",
            Severity::Warning => "Warning",
            Severity::Info => "Info",
        }
    }
}

impl Default for HtmlOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for HtmlOutput {
    fn format(&self, report: &ScanReport) -> Result<String> {
        let mut output = String::new();

        // HTML header with embedded CSS
        output.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>env-audit Report</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            line-height: 1.6;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        h1 { color: #333; border-bottom: 2px solid #333; padding-bottom: 10px; }
        h2 { color: #555; margin-top: 30px; }
        .summary { display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 30px; }
        .stat-card {
            background: white;
            border-radius: 8px;
            padding: 15px 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .stat-card h3 { margin: 0 0 5px 0; color: #666; font-size: 0.9em; }
        .stat-card .value { font-size: 1.8em; font-weight: bold; color: #333; }
        table { width: 100%; border-collapse: collapse; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        th { background: #333; color: white; padding: 12px; text-align: left; }
        td { padding: 12px; border-bottom: 1px solid #eee; }
        tr:last-child td { border-bottom: none; }
        tr:hover { background: #f9f9f9; }
        .severity { display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 0.8em; font-weight: bold; }
        .error { background: #ffebee; color: #c62828; }
        .warning { background: #fff3e0; color: #ef6c00; }
        .info { background: #e3f2fd; color: #1565c0; }
        .var-name { font-family: monospace; background: #f5f5f5; padding: 2px 6px; border-radius: 4px; }
        .location { font-family: monospace; font-size: 0.9em; color: #666; }
        .success { background: #e8f5e9; color: #2e7d32; padding: 20px; border-radius: 8px; text-align: center; }
        footer { margin-top: 40px; text-align: center; color: #999; font-size: 0.9em; }
    </style>
</head>
<body>
    <h1>env-audit Report</h1>
"#);

        // Summary cards
        output.push_str(r#"    <div class="summary">"#);
        output.push_str(&format!(
            r#"
        <div class="stat-card">
            <h3>Files Scanned</h3>
            <div class="value">{}</div>
        </div>
        <div class="stat-card">
            <h3>Env Files</h3>
            <div class="value">{}</div>
        </div>
        <div class="stat-card">
            <h3>Vars Defined</h3>
            <div class="value">{}</div>
        </div>
        <div class="stat-card">
            <h3>Vars Used</h3>
            <div class="value">{}</div>
        </div>
        <div class="stat-card">
            <h3>Errors</h3>
            <div class="value" style="color: #c62828">{}</div>
        </div>
        <div class="stat-card">
            <h3>Warnings</h3>
            <div class="value" style="color: #ef6c00">{}</div>
        </div>
"#,
            report.summary.files_scanned,
            report.summary.env_files_found,
            report.summary.vars_defined,
            report.summary.vars_used,
            report.summary.errors,
            report.summary.warnings,
        ));
        output.push_str("    </div>\n");

        if report.issues.is_empty() {
            output.push_str(r#"    <div class="success"><h2>No issues found!</h2></div>"#);
        } else {
            // Group issues
            let missing: Vec<_> = report.issues.iter()
                .filter(|i| i.kind == IssueKind::MissingEnvVar)
                .collect();
            let unused: Vec<_> = report.issues.iter()
                .filter(|i| i.kind == IssueKind::UnusedEnvVar)
                .collect();
            let naming: Vec<_> = report.issues.iter()
                .filter(|i| i.kind == IssueKind::InconsistentNaming)
                .collect();

            // Missing vars table
            if !missing.is_empty() {
                output.push_str("    <h2>Missing Environment Variables</h2>\n");
                output.push_str("    <table>\n");
                output.push_str("        <tr><th>Severity</th><th>Variable</th><th>Used In</th></tr>\n");
                for issue in &missing {
                    let locations: String = issue.locations.iter()
                        .take(3)
                        .map(|l| format!("<span class=\"location\">{}</span>", l))
                        .collect::<Vec<_>>()
                        .join("<br>");
                    output.push_str(&format!(
                        "        <tr><td><span class=\"severity {}\">{}</span></td><td class=\"var-name\">{}</td><td>{}</td></tr>\n",
                        Self::severity_class(issue.severity),
                        Self::severity_label(issue.severity),
                        issue.var_name,
                        locations
                    ));
                }
                output.push_str("    </table>\n");
            }

            // Unused vars table
            if !unused.is_empty() {
                output.push_str("    <h2>Unused Environment Variables</h2>\n");
                output.push_str("    <table>\n");
                output.push_str("        <tr><th>Severity</th><th>Variable</th><th>Defined In</th></tr>\n");
                for issue in &unused {
                    let locations: String = issue.locations.iter()
                        .map(|l| format!("<span class=\"location\">{}</span>", l))
                        .collect::<Vec<_>>()
                        .join("<br>");
                    output.push_str(&format!(
                        "        <tr><td><span class=\"severity {}\">{}</span></td><td class=\"var-name\">{}</td><td>{}</td></tr>\n",
                        Self::severity_class(issue.severity),
                        Self::severity_label(issue.severity),
                        issue.var_name,
                        locations
                    ));
                }
                output.push_str("    </table>\n");
            }

            // Naming issues table
            if !naming.is_empty() {
                output.push_str("    <h2>Naming Convention Issues</h2>\n");
                output.push_str("    <table>\n");
                output.push_str("        <tr><th>Severity</th><th>Variable</th><th>Suggestion</th></tr>\n");
                for issue in &naming {
                    output.push_str(&format!(
                        "        <tr><td><span class=\"severity {}\">{}</span></td><td class=\"var-name\">{}</td><td>{}</td></tr>\n",
                        Self::severity_class(issue.severity),
                        Self::severity_label(issue.severity),
                        issue.var_name,
                        issue.suggestion.as_deref().unwrap_or("")
                    ));
                }
                output.push_str("    </table>\n");
            }
        }

        // Footer
        output.push_str(&format!(
            r#"
    <footer>
        Generated by env-audit | Scan duration: {}ms
    </footer>
</body>
</html>
"#,
            report.scan_duration_ms
        ));

        Ok(output)
    }
}
