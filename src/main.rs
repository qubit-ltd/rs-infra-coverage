// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Command-line entry point for the coverage utility.

use std::fs;
use std::process;

use anyhow::Result;
use clap::Parser;
use qubit_infra_coverage::check;
use qubit_infra_coverage::clippy;
use qubit_infra_coverage::collect;
use qubit_infra_coverage::collect_with_threshold_policy;
use qubit_infra_coverage::report;
use qubit_infra_coverage::resolve_config_path;

use crate::cli::Cli;
use crate::command::Command;

mod cli;
mod command;

/// Parses CLI arguments, resolves the project configuration, and runs the
/// selected command. Parse failures and command failures are reported to
/// stderr before the process exits with the corresponding nonzero status.
fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let status = error.exit_code();
            let _ = error.print();
            if status == 0 {
                println!("✅ rs-infra-coverage: help succeeded");
            } else {
                eprintln!("❌ rs-infra-coverage: failed (exit code {status})");
            }
            process::exit(status);
        }
    };
    match run(cli) {
        Ok(command) => println!("✅ rs-infra-coverage: {command} succeeded"),
        Err(error) => {
            eprintln!("❌ rs-infra-coverage: failed: {error:#}");
            process::exit(1);
        }
    }
}

/// Parses arguments and executes the selected command.
///
/// # Parameters
///
/// * `cli` - Parsed command-line options.
fn run(cli: Cli) -> Result<&'static str> {
    let project = fs::canonicalize(&cli.project)?;
    let config_path = resolve_config_path(&project, &cli.config)?;
    let command = match cli.command {
        Command::Collect {
            output,
            ignore_thresholds,
        } => {
            if ignore_thresholds {
                collect_with_threshold_policy(&project, &config_path, output.as_deref(), false)?;
            } else {
                collect(&project, &config_path, output.as_deref())?;
            }
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
