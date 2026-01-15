use std::collections::HashSet;

use crate::types::{EnvVarDefinition, EnvVarUsage, Issue, IssueKind, Location, Severity};

/// Find environment variables that are defined in .env but never used in code
pub fn find_unused_vars(definitions: &[EnvVarDefinition], usages: &[EnvVarUsage]) -> Vec<Issue> {
    // Collect all defined var names
    let defined_names: HashSet<&str> = definitions.iter().map(|d| d.name.as_str()).collect();

    // Collect all used var names
    let used_names: HashSet<&str> = usages.iter().map(|u| u.name.as_str()).collect();

    // Find vars that are defined but not used
    let unused_names: Vec<&str> = defined_names.difference(&used_names).copied().collect();

    // Create issues for each unused var
    let mut issues = Vec::new();
    for name in unused_names {
        // Find all locations where this var is defined
        let locations: Vec<Location> = definitions
            .iter()
            .filter(|d| d.name == name)
            .map(|d| Location {
                file: d.source_file.clone(),
                line: Some(d.line),
                column: None,
            })
            .collect();

        let message = format!("'{}' is defined but never used in code", name);

        issues.push(Issue {
            kind: IssueKind::UnusedEnvVar,
            severity: Severity::Warning,
            var_name: name.to_string(),
            message,
            locations,
            suggestion: Some(format!(
                "Remove {} from your .env file if it's no longer needed",
                name
            )),
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Language;
    use std::path::PathBuf;

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
    fn test_no_unused_vars() {
        let definitions = vec![make_definition("API_KEY")];
        let usages = vec![make_usage("API_KEY")];

        let issues = find_unused_vars(&definitions, &usages);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_unused_var() {
        let definitions = vec![make_definition("API_KEY"), make_definition("OLD_KEY")];
        let usages = vec![make_usage("API_KEY")];

        let issues = find_unused_vars(&definitions, &usages);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].var_name, "OLD_KEY");
        assert_eq!(issues[0].kind, IssueKind::UnusedEnvVar);
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn test_all_unused() {
        let definitions = vec![make_definition("API_KEY"), make_definition("DATABASE_URL")];
        let usages = vec![];

        let issues = find_unused_vars(&definitions, &usages);
        assert_eq!(issues.len(), 2);
    }
}
