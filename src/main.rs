// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Command-line entry point for the coverage utility.

use std::fs;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::Parser;
use clap::Subcommand;

use qubit_infra_coverage::check;
use qubit_infra_coverage::clippy;
use qubit_infra_coverage::collect;
use qubit_infra_coverage::report;
use qubit_infra_coverage::resolve_config_path;

/// Defines the top-level command-line options.
#[derive(Debug, Parser)]
#[command(name = "rs-infra-coverage")]
struct Cli {
    /// Project root in which coverage commands run.
    #[arg(long, default_value = ".")]
    project: PathBuf,
    /// Coverage configuration path, relative to the project unless absolute.
    #[arg(long, default_value = ".infra/ci/coverage.json")]
    config: PathBuf,
    /// Operation to perform.
    #[command(subcommand)]
    command: Command,
}

/// Selects the coverage operation to execute.
#[derive(Debug, Subcommand)]
enum Command {
    /// Collect and validate a coverage report.
    Collect {
        /// Optional output path for the raw report.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Validate a coverage report against configured thresholds.
    Check {
        /// Input LLVM coverage JSON report.
        #[arg(long)]
        input: PathBuf,
    },
    /// Print selected coverage metrics without threshold validation.
    Report {
        /// Input LLVM coverage JSON report.
        #[arg(long)]
        input: PathBuf,
    },
    /// Run Clippy with optional coverage cfg support.
    Clippy {
        /// Enable `RUSTFLAGS=--cfg coverage`.
        #[arg(long)]
        coverage_cfg: bool,
    },
}

/// Parses CLI arguments, resolves the project configuration, and runs a command.
///
/// # Errors
///
/// Returns an error when the project cannot be canonicalized, configuration
/// resolution fails, or the selected coverage operation fails.
fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let status = error.exit_code();
            let _ = error.print();
            if status == 0 {
                println!("rs-infra-coverage: help succeeded");
            } else {
                eprintln!("rs-infra-coverage: failed (exit code {status})");
            }
            process::exit(status);
        }
    };
    match run(cli) {
        Ok(command) => println!("rs-infra-coverage: {command} succeeded"),
        Err(error) => {
            eprintln!("rs-infra-coverage: failed: {error:#}");
            process::exit(1);
        }
    }
}

/// Parses arguments and executes the selected command.
fn run(cli: Cli) -> Result<&'static str> {
    let project = fs::canonicalize(&cli.project)?;
    let configured_path = if cli.config.is_absolute() {
        cli.config
    } else {
        project.join(cli.config)
    };
    let config_path = resolve_config_path(&project, &configured_path)?;
    let command = match cli.command {
        Command::Collect { output } => {
            collect(&project, &config_path, output.as_deref())?;
            "collect"
        }
        Command::Check { input } => {
            check(&project, &config_path, &input)?;
            "check"
        }
        Command::Report { input } => {
            report(&project, &config_path, &input)?;
            "report"
        }
        Command::Clippy { coverage_cfg } => {
            clippy(&project, &config_path, coverage_cfg)?;
            "clippy"
        }
    };
    Ok(command)
}
