mod missing;
mod naming;
mod unused;

pub use missing::find_missing_vars;
pub use naming::find_naming_issues;
pub use unused::find_unused_vars;

use crate::config::Config;
use crate::rules::get_all_rules;
use crate::types::{EnvVarDefinition, EnvVarUsage, Issue};

/// Run all analyses and return combined issues
pub fn analyze(
    definitions: &[EnvVarDefinition],
    usages: &[EnvVarUsage],
    config: &Config,
) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Find missing env vars (used but not defined)
    issues.extend(find_missing_vars(definitions, usages));

    // Find unused env vars (defined but not used)
    issues.extend(find_unused_vars(definitions, usages));

    // Find naming convention issues
    let rules = get_all_rules(config);
    issues.extend(find_naming_issues(
        definitions,
        usages,
        &rules,
        &config.naming.ignore_patterns,
    ));

    // Sort by severity (errors first) then by var name
    issues.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.var_name.cmp(&b.var_name))
    });

    issues
}
