mod builtin;

pub use builtin::get_builtin_rules;

use crate::config::Config;
use crate::types::Severity;

/// A naming convention rule
#[derive(Debug, Clone)]
pub struct NamingRule {
    pub name: String,
    pub description: Option<String>,
    pub alternatives: Vec<String>,
    pub preferred: String,
    pub severity: Severity,
}

/// Get all naming rules (built-in + custom from config)
pub fn get_all_rules(config: &Config) -> Vec<NamingRule> {
    let mut rules = Vec::new();

    // Add built-in rules if enabled
    if config.naming.builtin_rules {
        rules.extend(get_builtin_rules());
    }

    // Add custom rules from config
    for custom in &config.naming.custom_rules {
        rules.push(NamingRule {
            name: custom.name.clone(),
            description: custom.description.clone(),
            alternatives: custom.alternatives.clone(),
            preferred: custom.preferred.clone(),
            severity: custom.severity_level(),
        });
    }

    rules
}
