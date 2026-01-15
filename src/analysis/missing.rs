use std::collections::HashSet;

use crate::types::{EnvVarDefinition, EnvVarUsage, Issue, IssueKind, Location, Severity};

/// Find environment variables that are used in code but not defined in any .env file
pub fn find_missing_vars(
    definitions: &[EnvVarDefinition],
    usages: &[EnvVarUsage],
) -> Vec<Issue> {
    // Collect all defined var names
    let defined_names: HashSet<&str> = definitions
        .iter()
        .map(|d| d.name.as_str())
        .collect();

    // Collect all used var names
    let used_names: HashSet<&str> = usages
        .iter()
        .map(|u| u.name.as_str())
        .collect();

    // Find vars that are used but not defined
    let missing_names: Vec<&str> = used_names
        .difference(&defined_names)
        .copied()
        .collect();

    // Create issues for each missing var
    let mut issues = Vec::new();
    for name in missing_names {
        // Find all locations where this var is used
        let locations: Vec<Location> = usages
            .iter()
            .filter(|u| u.name == name)
            .map(|u| Location {
                file: u.file_path.clone(),
                line: Some(u.line),
                column: Some(u.column),
            })
            .collect();

        let location_count = locations.len();
        let message = if location_count == 1 {
            format!("'{}' is used in code but not defined in any .env file", name)
        } else {
            format!(
                "'{}' is used in {} locations but not defined in any .env file",
                name, location_count
            )
        };

        issues.push(Issue {
            kind: IssueKind::MissingEnvVar,
            severity: Severity::Error,
            var_name: name.to_string(),
            message,
            locations,
            suggestion: Some(format!("Add {} to your .env file", name)),
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::types::Language;

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

    #[test]
    fn test_no_missing_vars() {
        let definitions = vec![make_definition("API_KEY")];
        let usages = vec![make_usage("API_KEY")];

        let issues = find_missing_vars(&definitions, &usages);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_missing_var() {
        let definitions = vec![make_definition("API_KEY")];
        let usages = vec![make_usage("API_KEY"), make_usage("DATABASE_URL")];

        let issues = find_missing_vars(&definitions, &usages);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].var_name, "DATABASE_URL");
        assert_eq!(issues[0].kind, IssueKind::MissingEnvVar);
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn test_multiple_missing_vars() {
        let definitions = vec![];
        let usages = vec![make_usage("API_KEY"), make_usage("DATABASE_URL")];

        let issues = find_missing_vars(&definitions, &usages);
        assert_eq!(issues.len(), 2);
    }
}
