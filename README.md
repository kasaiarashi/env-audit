# env-audit

A fast CLI tool that scans your project for environment variable issues.

## Features

- **Missing env vars** - Detects variables used in code but not defined in `.env` files
- **Unused env vars** - Detects variables defined in `.env` but never used in code
- **Inconsistent naming** - Flags naming conflicts like `DB_URL` vs `DATABASE_URL`

## Supported Languages

- JavaScript / TypeScript
- Python
- Rust
- Go
- Ruby
- PHP
- Java
- C#

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/env-audit
```

## Usage

```bash
# Scan current directory
env-audit

# Scan a specific project
env-audit -p /path/to/project

# Output as JSON
env-audit -f json

# CI mode (exits with code 1 if errors found)
env-audit check

# Generate config file
env-audit init

# List all env vars
env-audit list

# Compare two env files
env-audit compare .env .env.example
```

## Output Formats

| Format | Flag | Description |
|--------|------|-------------|
| Terminal | `-f terminal` | Colored tables (default) |
| JSON | `-f json` | Machine-readable output |
| Markdown | `-f markdown` | Report format |
| HTML | `-f html` | Styled HTML report |

## Configuration

Create a `.env-audit.toml` file in your project root:

```toml
[scan]
env_files = [".env", ".env.local", ".env.example"]
exclude = ["**/node_modules/**", "**/target/**", "**/vendor/**"]

[naming]
builtin_rules = true
ignore_patterns = ["^_", "^INTERNAL_"]

# Custom naming rules
[[naming.custom_rules]]
name = "database-url"
alternatives = ["DB_URL", "DB_CONNECTION"]
preferred = "DATABASE_URL"
severity = "warning"

[output]
format = "terminal"
min_severity = "info"
```

## Built-in Naming Rules

| Alternatives | Preferred | Severity |
|--------------|-----------|----------|
| `DB_URL`, `DB_HOST` | `DATABASE_URL` | warning |
| `REDIS_HOST` | `REDIS_URL` | warning |
| `APIKEY` | `API_KEY` | info |
| `SECRET`, `APP_SECRET` | `SECRET_KEY` | info |
| `APP_PORT`, `SERVER_PORT` | `PORT` | info |

## Example Output

```
env-audit scan results

Files scanned: 847  |  Env files: 3  |  Duration: 127ms
Vars defined: 12  |  Vars used: 15

MISSING ENV VARS (3)
+---+--------------+---------------------------+
|   | Variable     | Used In                   |
+===+==============+===========================+
| x | API_KEY      | src/services/api.js:12    |
| x | REDIS_URL    | lib/cache.py:8            |
| x | SENTRY_DSN   | src/config.rs:23          |
+---+--------------+---------------------------+

UNUSED ENV VARS (2)
+---+----------------+---------------------------+
|   | Variable       | Defined In                |
+===+================+===========================+
| ! | OLD_API_KEY    | .env:15                   |
| ! | DEBUG_MODE     | .env.local:3              |
+---+----------------+---------------------------+

SUMMARY
  Errors: 3  |  Warnings: 2  |  Info: 0
```

## CI Integration

Use the `check` command in your CI pipeline:

```yaml
# GitHub Actions
- name: Check env vars
  run: env-audit check --fail-on error
```

Exit codes:
- `0` - No issues at or above the specified severity
- `1` - Issues found at or above the specified severity

## License

MIT
