use regex::Regex;
use std::collections::HashSet;

use crate::rules::NamingRule;
use crate::types::{EnvVarDefinition, EnvVarUsage, Issue, IssueKind, Location};

/// Find environment variables with inconsistent naming
pub fn find_naming_issues(
    definitions: &[EnvVarDefinition],
    usages: &[EnvVarUsage],
    rules: &[NamingRule],
    ignore_patterns: &[String],
) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Compile ignore patterns
    let ignore_regexes: Vec<Regex> = ignore_patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect();

    // Collect all var names (both defined and used)
    let all_names: HashSet<&str> = definitions
        .iter()
        .map(|d| d.name.as_str())
        .chain(usages.iter().map(|u| u.name.as_str()))
        .collect();

    // Check each rule
    for rule in rules {
        // Find which alternatives from this rule are present
        let present_alternatives: Vec<&str> = rule
            .alternatives
            .iter()
            .filter(|alt| all_names.contains(alt.as_str()))
            .map(|s| s.as_str())
            .collect();

        // If any alternatives are present, suggest using the preferred name
        for alt_name in present_alternatives {
            // Skip if the name matches an ignore pattern
            if ignore_regexes.iter().any(|re| re.is_match(alt_name)) {
                continue;
            }

            // Collect locations from both definitions and usages
            let mut locations: Vec<Location> = Vec::new();

            for def in definitions.iter().filter(|d| d.name == alt_name) {
                locations.push(Location {
                    file: def.source_file.clone(),
                    line: Some(def.line),
                    column: None,
                });
            }

            for usage in usages.iter().filter(|u| u.name == alt_name) {
                locations.push(Location {
                    file: usage.file_path.clone(),
                    line: Some(usage.line),
                    column: Some(usage.column),
                });
            }

            // Deduplicate locations by file and line
            locations.sort_by(|a, b| {
                a.file.cmp(&b.file)
                    .then_with(|| a.line.cmp(&b.line))
            });
            locations.dedup_by(|a, b| a.file == b.file && a.line == b.line);

            let message = format!(
                "'{}' could be renamed to '{}' for consistency",
                alt_name, rule.preferred
            );

            issues.push(Issue {
                kind: IssueKind::InconsistentNaming,
                severity: rule.severity,
                var_name: alt_name.to_string(),
                message,
                locations,
                suggestion: Some(format!(
                    "Consider using '{}' instead of '{}'{}",
                    rule.preferred,
                    alt_name,
                    rule.description
                        .as_ref()
                        .map(|d| format!(" ({})", d))
                        .unwrap_or_default()
                )),
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::types::{Language, Severity};

    fn make_definition(name: &str) -> EnvVarDefinition {
        EnvVarDefinition {
            name: name.to_string(),
            value: Some("test".to_string()),
            source_file: PathBuf::from(".env"),
            line: 1,
        }
    }

    fn make_usage(name: &str) -> EnvVarUsage {
        EnvVarUsage {
            name: name.to_string(),
            file_path: PathBuf::from("src/app.js"),
            line: 10,
            column: 5,
            language: Language::JavaScript,
            context: None,
        }
    }

    fn make_rule(alternatives: Vec<&str>, preferred: &str) -> NamingRule {
        NamingRule {
            name: "test-rule".to_string(),
            description: Some("Test rule".to_string()),
            alternatives: alternatives.into_iter().map(String::from).collect(),
            preferred: preferred.to_string(),
            severity: Severity::Warning,
        }
    }

    #[test]
    fn test_no_naming_issues() {
        let definitions = vec![make_definition("DATABASE_URL")];
        let usages = vec![make_usage("DATABASE_URL")];
        let rules = vec![make_rule(vec!["DB_URL", "DB_HOST"], "DATABASE_URL")];

        let issues = find_naming_issues(&definitions, &usages, &rules, &[]);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_naming_issue_found() {
        let definitions = vec![make_definition("DB_URL")];
        let usages = vec![make_usage("DB_URL")];
        let rules = vec![make_rule(vec!["DB_URL", "DB_HOST"], "DATABASE_URL")];

        let issues = find_naming_issues(&definitions, &usages, &rules, &[]);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].var_name, "DB_URL");
        assert_eq!(issues[0].kind, IssueKind::InconsistentNaming);
    }

    #[test]
    fn test_ignore_pattern() {
        let definitions = vec![make_definition("_INTERNAL_VAR")];
        let usages = vec![];
        let rules = vec![make_rule(vec!["_INTERNAL_VAR"], "INTERNAL_VAR")];
        let ignore = vec!["^_".to_string()];

        let issues = find_naming_issues(&definitions, &usages, &rules, &ignore);
        assert!(issues.is_empty());
    }
}
