// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Configuration models for coverage collection and threshold checking.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::ClippyConfig;
use crate::Thresholds;

/// Describes the coverage policy and collection scope for a project.
///
/// Missing fields use the tool's defaults. Unknown fields are rejected when a
/// configuration file is loaded.
///
/// # Examples
///
/// ```
/// use qubit_infra_coverage::Config;
///
/// let config = Config::default();
/// assert_eq!(config.scope, None);
/// ```
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Selects `default-members`, `workspace`, or `package` collection.
    pub scope: Option<String>,
    /// Lists workspace packages excluded from collection.
    #[serde(default)]
    pub exclude_packages: Vec<String>,
    /// Maps package names to source directory paths included in thresholds.
    #[serde(default)]
    pub source_dirs: BTreeMap<String, Vec<String>>,
    /// Maps package names to source paths excluded from threshold checks.
    #[serde(default)]
    pub threshold_exempt_files: BTreeMap<String, Vec<String>>,
    /// Defines the minimum coverage percentages.
    #[serde(default)]
    pub thresholds: Thresholds,
    /// Configures whether the coverage cfg is used for Clippy.
    #[serde(default)]
    pub clippy: ClippyConfig,
    /// Enables coverage cfg for Clippy using the legacy top-level setting.
    #[serde(default, alias = "run_coverage_cfg_clippy")]
    pub coverage_cfg_clippy: bool,
}
