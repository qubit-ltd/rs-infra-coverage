// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

const TOOLS_REPOSITORY_HTTPS: &str = "https://github.com/qubit-ltd/rs-infra-tools.git";

#[test]
fn remote_ci_bootstrap_uses_https_for_public_tools_repository() {
    let updater = include_str!("../update-infra.sh");
    let bootstrap = include_str!("../.infra/bootstrap.sh");
    let snapshot = include_str!("../.infra/bootstrap-source.json");

    assert!(updater.contains(&format!("repository={TOOLS_REPOSITORY_HTTPS}")));
    assert!(bootstrap.contains(&format!("manager_repo={TOOLS_REPOSITORY_HTTPS}")));
    assert!(snapshot.contains(&format!("\"source_repository\": \"{TOOLS_REPOSITORY_HTTPS}\"")));
    assert!(!updater.contains("git@github.com:"));
    assert!(!bootstrap.contains("git@github.com:"));
    assert!(!snapshot.contains("git@github.com:"));
}
