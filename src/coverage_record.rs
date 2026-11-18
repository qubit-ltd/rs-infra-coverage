// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::metric_counts::MetricCounts;
use crate::file_coverage::FileCoverage;

/// Retains source paths and hit counts for one LLVM file summary.
pub(super) struct CoverageRecord {
    /// Display metrics and source path for this file.
    pub(super) coverage: FileCoverage,
    /// Line counts reported by LLVM.
    pub(super) lines: Option<MetricCounts>,
    /// Function counts reported by LLVM.
    pub(super) functions: Option<MetricCounts>,
    /// Region counts reported by LLVM.
    pub(super) regions: Option<MetricCounts>,
    /// Branch counts reported by LLVM.
    pub(super) branches: Option<MetricCounts>,
}
