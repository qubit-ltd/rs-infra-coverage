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
    /// The path of the covered source file.
    pub filename: String,
    /// The percentage of covered lines, if reported.
    pub lines_percent: Option<f64>,
    /// The percentage of covered functions, if reported.
    pub functions_percent: Option<f64>,
    /// The percentage of covered regions, if reported.
    pub regions_percent: Option<f64>,
    /// The percentage of covered branches, if reported.
    pub branches_percent: Option<f64>,
}
