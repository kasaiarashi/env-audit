use super::NamingRule;
use crate::types::Severity;

/// Returns the built-in naming convention rules
pub fn get_builtin_rules() -> Vec<NamingRule> {
    vec![
        NamingRule {
            name: "database-url".to_string(),
            description: Some("Database connection URL".to_string()),
            alternatives: vec![
                "DB_URL".to_string(),
                "DB_CONNECTION".to_string(),
                "DB_HOST".to_string(),
            ],
            preferred: "DATABASE_URL".to_string(),
            severity: Severity::Warning,
        },
        NamingRule {
            name: "redis-url".to_string(),
            description: Some("Redis connection URL".to_string()),
            alternatives: vec!["REDIS_HOST".to_string(), "REDIS_CONNECTION".to_string()],
            preferred: "REDIS_URL".to_string(),
            severity: Severity::Warning,
        },
        NamingRule {
            name: "api-key".to_string(),
            description: Some("API key naming".to_string()),
            alternatives: vec!["APIKEY".to_string(), "API_SECRET".to_string()],
            preferred: "API_KEY".to_string(),
            severity: Severity::Info,
        },
        NamingRule {
            name: "secret-key".to_string(),
            description: Some("Secret key naming".to_string()),
            alternatives: vec!["SECRET".to_string(), "APP_SECRET".to_string()],
            preferred: "SECRET_KEY".to_string(),
            severity: Severity::Info,
        },
        NamingRule {
            name: "port".to_string(),
            description: Some("Application port".to_string()),
            alternatives: vec![
                "APP_PORT".to_string(),
                "SERVER_PORT".to_string(),
                "HTTP_PORT".to_string(),
            ],
            preferred: "PORT".to_string(),
            severity: Severity::Info,
        },
        NamingRule {
            name: "log-level".to_string(),
            description: Some("Logging level".to_string()),
            alternatives: vec!["LOGLEVEL".to_string(), "LOGGING_LEVEL".to_string()],
            preferred: "LOG_LEVEL".to_string(),
            severity: Severity::Info,
        },
        NamingRule {
            name: "aws-region".to_string(),
            description: Some("AWS region".to_string()),
            alternatives: vec!["REGION".to_string(), "AMAZON_REGION".to_string()],
            preferred: "AWS_REGION".to_string(),
            severity: Severity::Info,
        },
        NamingRule {
            name: "jwt-secret".to_string(),
            description: Some("JWT signing secret".to_string()),
            alternatives: vec!["JWT_KEY".to_string(), "TOKEN_SECRET".to_string()],
            preferred: "JWT_SECRET".to_string(),
            severity: Severity::Info,
        },
    ]
}
