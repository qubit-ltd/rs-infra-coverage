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
/// The default requires 90% lines, 95% functions, and 85% regions coverage;
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
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    /// Minimum line coverage percentage, if configured.
    pub lines: Option<f64>,
    /// Minimum function coverage percentage, if configured.
    pub functions: Option<f64>,
    /// Minimum region coverage percentage, if configured.
    pub regions: Option<f64>,
    /// Minimum branch coverage percentage, if configured.
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
