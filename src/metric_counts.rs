// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Stores the covered and total item counts for one coverage metric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MetricCounts {
    /// Number of covered items reported by LLVM.
    pub(super) covered: u64,
    /// Total number of items measured by LLVM.
    pub(super) count: u64,
}
