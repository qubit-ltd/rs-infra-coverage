// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fs;
use std::path::Path;

use syn::spanned::Spanned;
use syn::visit::Visit;

/// Collects source extents for inline test modules.
#[derive(Default)]
struct TestModules {
    /// Source line ranges occupied by inline test modules.
    ranges: Vec<(usize, usize)>,
}

impl<'ast> Visit<'ast> for TestModules {
    /// Visits a module and records it when guarded by `cfg(test)`.
    ///
    /// # Parameters
    ///
    /// * `item` - Source module being inspected.
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        let is_test = item.attrs.iter().any(|attribute| {
            attribute.path().is_ident("cfg")
                && attribute
                    .meta
                    .require_list()
                    .is_ok_and(|list| list.tokens.to_string().trim() == "test")
        });
        if is_test && let Some((_, items)) = &item.content {
            let start = item.span().start().line;
            let end = items
                .last()
                .map_or(item.span().end().line, |last| last.span().end().line);
            self.ranges.push((start, end));
        }
        syn::visit::visit_item_mod(self, item);
    }
}

/// Finds inline `#[cfg(test)]` module source ranges in a Rust file.
///
/// # Parameters
///
/// * `path` - Rust source file whose inline test modules are inspected.
pub(super) fn test_module_ranges(path: &Path) -> Vec<(usize, usize)> {
    let Ok(source) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(file) = syn::parse_file(&source) else {
        return Vec::new();
    };
    let mut visitor = TestModules::default();
    visitor.visit_file(&file);
    visitor.ranges
}
// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
