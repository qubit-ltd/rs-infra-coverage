// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::path::PathBuf;

use clap::Parser;

use crate::command::Command;

/// Defines the top-level command-line options.
#[derive(Debug, Parser)]
#[command(name = "rs-infra-coverage")]
pub(crate) struct Cli {
    /// Project root in which coverage commands run.
    #[arg(long, default_value = ".")]
    pub(crate) project: PathBuf,
    /// Coverage configuration path, relative to the project unless absolute.
    #[arg(long, default_value = ".infra/coverage/coverage.json")]
    pub(crate) config: PathBuf,
    /// Operation to perform.
    #[command(subcommand)]
    pub(crate) command: Command,
}
