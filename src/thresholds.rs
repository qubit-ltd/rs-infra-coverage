// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Coverage threshold configuration.

use serde::Deserialize;

/// Stores minimum acceptable percentages for each LLVM coverage metric.
///
/// The default requires >90% lines, >=95% functions, and >85% regions coverage;
/// branch coverage has no default threshold.
///
/// # Examples
///
/// ```
/// use qubit_infra_coverage::Thresholds;
///
/// let thresholds = Thresholds::default();
/// assert_eq!(thresholds.lines, Some(90.0));
/// ```
#[derive(Debug, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct Thresholds {
    /// Exclusive minimum line coverage percentage. `None` is rejected.
    ///
    /// When configured, the value must be between `0.0` and `100.0`.
    pub lines: Option<f64>,
    /// Inclusive minimum function coverage percentage. `None` is rejected.
    ///
    /// The value must be between `0.0` and `100.0`.
    pub functions: Option<f64>,
    /// Exclusive minimum region coverage percentage. `None` is rejected.
    ///
    /// When configured, the value must be between `0.0` and `100.0`.
    pub regions: Option<f64>,
    /// Minimum branch coverage percentage, or `None` to skip branch checking.
    ///
    /// When configured, the value must be between `0.0` and `100.0`.
    pub branches: Option<f64>,
}

impl Default for Thresholds {
    /// Creates the default line, function, and region thresholds.
    fn default() -> Self {
        Self {
            lines: Some(90.0),
            functions: Some(95.0),
            regions: Some(85.0),
            branches: None,
        }
    }
}
