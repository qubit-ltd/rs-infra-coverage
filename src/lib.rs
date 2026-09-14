// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Coverage collection, validation, and reporting.

mod clippy_config;
mod config;
mod coverage;
mod file_coverage;
mod thresholds;

pub use clippy_config::ClippyConfig;
pub use config::Config;
pub use coverage::check;
pub use coverage::clippy;
pub use coverage::collect;
pub use coverage::load_config;
pub use coverage::report;
pub use coverage::resolve_config_path;
pub use file_coverage::FileCoverage;
pub use thresholds::Thresholds;
