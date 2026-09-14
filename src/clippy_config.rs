// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Clippy coverage configuration.

use serde::Deserialize;

/// Configures coverage-specific behavior for Clippy.
///
/// # Examples
///
/// ```
/// use qubit_infra_coverage::ClippyConfig;
///
/// let config = ClippyConfig::default();
/// assert!(!config.coverage_cfg);
/// ```
#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClippyConfig {
    /// Enables `RUSTFLAGS=--cfg coverage` for the child Clippy process.
    ///
    /// This affects only the Clippy invocation and does not change the
    /// environment of the current process or its caller.
    #[serde(alias = "run_coverage_cfg", alias = "run_coverage_cfg_clippy")]
    pub coverage_cfg: bool,
}
