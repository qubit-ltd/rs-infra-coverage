// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Coverage metrics for one source file.

/// Contains the coverage percentages reported for one source file.
///
/// A metric is `None` when the input report does not provide that metric.
///
/// # Examples
///
/// ```
/// use qubit_infra_coverage::FileCoverage;
///
/// let file = FileCoverage {
///     filename: "src/lib.rs".to_owned(),
///     lines_percent: Some(100.0),
///     functions_percent: Some(100.0),
///     regions_percent: Some(100.0),
///     branches_percent: None,
/// };
/// assert_eq!(file.lines_percent, Some(100.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FileCoverage {
    /// The path of the covered source file as reported by LLVM.
    ///
    /// The path may be absolute or project-relative; filtering resolves it
    /// against the project root when selecting threshold inputs.
    pub filename: String,
    /// The covered-line percentage in the inclusive `0.0..=100.0` range, if
    /// LLVM reported line metrics.
    pub lines_percent: Option<f64>,
    /// The covered-function percentage in the inclusive `0.0..=100.0` range,
    /// if LLVM reported function metrics.
    pub functions_percent: Option<f64>,
    /// The covered-region percentage in the inclusive `0.0..=100.0` range, if
    /// LLVM reported region metrics.
    pub regions_percent: Option<f64>,
    /// The covered-branch percentage in the inclusive `0.0..=100.0` range, if
    /// LLVM reported branch metrics.
    pub branches_percent: Option<f64>,
}
