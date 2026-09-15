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
    ///
    /// `None` uses `default-members`. The `package` option requires the
    /// project manifest to declare a package name.
    pub scope: Option<String>,
    /// Lists workspace packages excluded from collection.
    ///
    /// The names are passed to `cargo llvm-cov --exclude` and must be
    /// non-empty.
    #[serde(default)]
    pub exclude_packages: Vec<String>,
    /// Maps package names to relative source directory paths included in
    /// threshold selection.
    ///
    /// An empty map selects the conventional `src/` directory. Paths are
    /// interpreted relative to the project root and cannot contain `..`.
    #[serde(default)]
    pub source_dirs: BTreeMap<String, Vec<String>>,
    /// Maps package names to source paths excluded from threshold checks.
    ///
    /// Exemptions are matched against selected report paths before aggregate
    /// metrics are computed. Paths are relative to the project root and cannot
    /// contain `..`.
    #[serde(default)]
    pub threshold_exempt_files: BTreeMap<String, Vec<String>>,
    /// Defines the minimum coverage percentages for selected files.
    ///
    /// Thresholds are evaluated from aggregate covered and total hit counts;
    /// metrics without counts or a configured threshold are not checked.
    #[serde(default)]
    pub thresholds: Thresholds,
    /// Configures whether the coverage cfg is used for Clippy.
    ///
    /// This nested setting is equivalent to the legacy top-level setting
    /// [`Self::coverage_cfg_clippy`].
    #[serde(default)]
    pub clippy: ClippyConfig,
    /// Enables coverage cfg for Clippy using the legacy top-level setting.
    ///
    /// The nested [`Self::clippy`] setting is preferred for new files; either
    /// setting enables the flag when it is true.
    #[serde(default, alias = "run_coverage_cfg_clippy")]
    pub coverage_cfg_clippy: bool,
}
