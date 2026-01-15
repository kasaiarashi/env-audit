use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;

use env_audit::cli::{Cli, Commands, OutputFormat, ScanArgs};
use env_audit::config::Config;
use env_audit::scanner::{parse_env_file, CodeScanner, FileWalker};
use env_audit::analysis::analyze;
use env_audit::output::{OutputFormatter, TerminalOutput, JsonOutput, MarkdownOutput, HtmlOutput};
use env_audit::types::{ScanReport, Severity};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => cmd_init(&cli),
        Some(Commands::Scan(args)) => cmd_scan(&cli, args),
        Some(Commands::Check(args)) => cmd_check(&cli, args),
        Some(Commands::List(args)) => cmd_list(&cli, args),
        Some(Commands::Compare(args)) => cmd_compare(&cli, args),
        None => {
            // Default to scan command
            let args = ScanArgs::default();
            cmd_scan(&cli, &args)
        }
    }
}

fn cmd_init(cli: &Cli) -> Result<()> {
    let config_path = cli.path.join(".env-audit.toml");

    if config_path.exists() {
        anyhow::bail!("Config file already exists: {}", config_path.display());
    }

    let content = Config::generate_default();
    std::fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    println!("Created config file: {}", config_path.display());
    Ok(())
}

fn cmd_scan(cli: &Cli, _args: &ScanArgs) -> Result<()> {
    let report = run_scan(cli)?;

    let output = format_output(&report, cli)?;
    print!("{}", output);

    if let Some(output_path) = &cli.output {
        std::fs::write(output_path, &output)?;
        if !cli.quiet {
            eprintln!("Report written to: {}", output_path.display());
        }
    }

    Ok(())
}

fn cmd_check(cli: &Cli, args: &env_audit::cli::CheckArgs) -> Result<()> {
    let report = run_scan(cli)?;

    let fail_severity: Severity = args.fail_on.into();

    // Count issues at or above the fail severity
    let failing_issues = report.issues.iter()
        .filter(|i| i.severity >= fail_severity)
        .count();

    if args.summary {
        println!(
            "Errors: {}  Warnings: {}  Info: {}",
            report.summary.errors,
            report.summary.warnings,
            report.summary.infos
        );
    } else {
        let output = format_output(&report, cli)?;
        print!("{}", output);
    }

    if failing_issues > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn cmd_list(cli: &Cli, args: &env_audit::cli::ListArgs) -> Result<()> {
    let config = Config::load(&cli.config)?;
    let walker = FileWalker::new(&cli.path, &config.scan);

    if !args.used {
        // Show defined vars
        println!("Defined environment variables:\n");
        let env_files = walker.find_env_files(&config.scan.env_files)?;
        for env_file in env_files {
            let definitions = parse_env_file(&env_file)?;
            for def in definitions {
                if args.locations {
                    println!("  {} ({}:{})", def.name, def.source_file.display(), def.line);
                } else {
                    println!("  {}", def.name);
                }
            }
        }
    }

    if !args.defined {
        // Show used vars
        println!("\nUsed environment variables:\n");
        let scanner = CodeScanner::new();
        let source_files = walker.find_source_files()?;
        let usages = scanner.scan_files(&source_files);

        let mut seen = std::collections::HashSet::new();
        for usage in usages {
            if seen.insert(usage.name.clone()) {
                if args.locations {
                    println!("  {} ({}:{})", usage.name, usage.file_path.display(), usage.line);
                } else {
                    println!("  {}", usage.name);
                }
            }
        }
    }

    Ok(())
}

fn cmd_compare(cli: &Cli, args: &env_audit::cli::CompareArgs) -> Result<()> {
    let file1_path = cli.path.join(&args.file1);
    let file2_path = cli.path.join(&args.file2);

    let defs1 = parse_env_file(&file1_path)?;
    let defs2 = parse_env_file(&file2_path)?;

    let names1: std::collections::HashSet<_> = defs1.iter().map(|d| &d.name).collect();
    let names2: std::collections::HashSet<_> = defs2.iter().map(|d| &d.name).collect();

    let only_in_1: Vec<_> = names1.difference(&names2).collect();
    let only_in_2: Vec<_> = names2.difference(&names1).collect();
    let in_both: Vec<_> = names1.intersection(&names2).collect();

    println!("Comparing {} and {}\n", args.file1.display(), args.file2.display());

    if !only_in_1.is_empty() {
        println!("Only in {}:", args.file1.display());
        for name in only_in_1 {
            println!("  {}", name);
        }
        println!();
    }

    if !only_in_2.is_empty() {
        println!("Only in {}:", args.file2.display());
        for name in only_in_2 {
            println!("  {}", name);
        }
        println!();
    }

    println!("In both files: {} variables", in_both.len());

    if args.show_values {
        println!("\nValues comparison:");
        for name in in_both {
            let val1 = defs1.iter().find(|d| &d.name == *name).and_then(|d| d.value.as_ref());
            let val2 = defs2.iter().find(|d| &d.name == *name).and_then(|d| d.value.as_ref());
            if val1 != val2 {
                println!("  {} differs:", name);
                println!("    {}: {:?}", args.file1.display(), val1);
                println!("    {}: {:?}", args.file2.display(), val2);
            }
        }
    }

    Ok(())
}

fn run_scan(cli: &Cli) -> Result<ScanReport> {
    let start = Instant::now();

    // Load config
    let config = Config::load(&cli.config)?;

    // Set up file walker
    let walker = FileWalker::new(&cli.path, &config.scan);

    // Find and parse .env files
    let env_files = walker.find_env_files(&config.scan.env_files)?;
    let mut definitions = Vec::new();
    for env_file in &env_files {
        definitions.extend(parse_env_file(env_file)?);
    }

    // Find and scan source files
    let source_files = walker.find_source_files()?;
    let scanner = CodeScanner::new();
    let usages = scanner.scan_files(&source_files);

    // Run analysis
    let issues = analyze(&definitions, &usages, &config);

    // Build report
    let mut report = ScanReport::new();
    report.definitions = definitions;
    report.usages = usages;
    report.issues = issues;
    report.summary.files_scanned = source_files.len();
    report.summary.env_files_found = env_files.len();
    report.calculate_summary();
    report.scan_duration_ms = start.elapsed().as_millis() as u64;

    Ok(report)
}

fn format_output(report: &ScanReport, cli: &Cli) -> Result<String> {
    match cli.format {
        OutputFormat::Terminal => {
            let formatter = TerminalOutput::new(cli.no_color, true);
            formatter.format(report)
        }
        OutputFormat::Json => {
            let formatter = JsonOutput::new(true);
            formatter.format(report)
        }
        OutputFormat::Markdown => {
            let formatter = MarkdownOutput::new();
            formatter.format(report)
        }
        OutputFormat::Html => {
            let formatter = HtmlOutput::new();
            formatter.format(report)
        }
    }
}
