# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-15

### Added
- Initial release of env-audit CLI tool
- Three core analysis features:
  - Missing env vars: detects variables used in code but not defined in .env files
  - Unused env vars: detects variables defined in .env but never used in code
  - Inconsistent naming: flags naming conflicts (e.g., DB_URL vs DATABASE_URL)
- Support for 8 programming languages:
  - JavaScript/TypeScript (process.env, import.meta.env)
  - Python (os.environ, os.getenv)
  - Rust (env::var, env!, option_env!)
  - Go (os.Getenv, os.LookupEnv)
  - Ruby (ENV[])
  - PHP (getenv, $_ENV, Laravel env())
  - Java (System.getenv)
  - C# (Environment.GetEnvironmentVariable)
- Four output formats:
  - Terminal: colored tables with comfy-table
  - JSON: machine-readable output
  - Markdown: report format
  - HTML: styled HTML reports
- Configuration via .env-audit.toml
- Built-in naming convention rules:
  - DATABASE_URL vs DB_URL/DB_HOST
  - REDIS_URL vs REDIS_HOST
  - API_KEY vs APIKEY
  - SECRET_KEY vs SECRET/APP_SECRET
  - PORT vs APP_PORT/SERVER_PORT
  - And more
- Custom naming rules support
- Multiple commands:
  - `scan`: scan project for issues (default)
  - `check`: CI mode with exit codes
  - `init`: generate config file
  - `list`: list all detected env vars
  - `compare`: compare two env files
- Parallel file scanning with rayon for performance
- Gitignore-aware file traversal with ignore crate
- 39 unit tests covering all language scanners and analysis logic
- GitHub Actions workflows for CI and releases
- Cross-platform support: Linux, Windows, macOS (x64 and ARM)

### Features
- Fast parallel scanning of large codebases
- Zero false positives with language-specific regex patterns
- Configurable severity levels (error, warning, info)
- Context-aware issue detection with line and column numbers
- Deduplication of overlapping pattern matches
- Ignore patterns support (e.g., ignore vars starting with _)
- Flexible output options with file writing support
- CI-friendly with appropriate exit codes

[unreleased]: https://github.com/kasaiarashi/env-audit/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kasaiarashi/env-audit/releases/tag/v0.1.0
