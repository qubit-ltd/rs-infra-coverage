// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::path::PathBuf;

use clap::Subcommand;

/// Selects the coverage operation to execute.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Collect and validate a coverage report.
    Collect {
        /// Optional output path for the raw report.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Report threshold shortfalls without failing the collection.
        #[arg(long)]
        ignore_thresholds: bool,
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
