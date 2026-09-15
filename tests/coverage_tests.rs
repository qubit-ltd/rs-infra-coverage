//! Regression tests for the legacy coverage policy through the public API.

use std::fs;
use std::path::Path;
use std::process::Command;

use qubit_infra_coverage::check;
use qubit_infra_coverage::load_config;
use qubit_infra_coverage::report;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

/// Creates a dependency-free Cargo project with two real source directories.
fn project() -> TempDir {
    let directory = tempfile::tempdir().expect("temporary project");
    package(directory.path(), "demo");
    fs::create_dir_all(directory.path().join("src/extra")).expect("extra source root");
    fs::write(directory.path().join("src/extra/item.rs"), "").expect("extra source file");
    directory
}

/// Writes a minimal package so Cargo metadata resolves real package roots.
fn package(root: &Path, name: &str) {
    fs::create_dir_all(root.join("src")).expect("source directory");
    fs::write(root.join("src/lib.rs"), "").expect("library source");
    fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )
    .expect("package manifest");
}

/// Builds an LLVM file record with independent metric boundary values.
fn record(path: &str, lines: u64, functions: u64, regions: u64) -> Value {
    json!({"filename":path,"summary":{
        "lines":{"covered":lines,"count":100,"percent":lines},
        "functions":{"covered":functions,"count":100,"percent":functions},
        "regions":{"covered":regions,"count":100,"percent":regions}
    }})
}

/// Writes a configuration and report and returns the public check result.
fn evaluate(root: &Path, config: Value, files: Vec<Value>) -> anyhow::Result<()> {
    fs::write(root.join("coverage.json"), config.to_string()).expect("configuration");
    fs::write(
        root.join("report.json"),
        json!({"data":[{"files":files}]}).to_string(),
    )
    .expect("coverage report");
    check(root, &root.join("coverage.json"), &root.join("report.json"))
}

#[test]
fn test_lines_equal_threshold_fail() {
    let project = project();
    let error = evaluate(
        project.path(),
        json!({}),
        vec![record("src/lib.rs", 90, 100, 100)],
    )
    .expect_err("exactly 90 percent lines must fail");
    assert!(error.to_string().contains("lines"), "{error:#}");
}

#[test]
fn test_regions_equal_threshold_fail() {
    let project = project();
    let error = evaluate(
        project.path(),
        json!({}),
        vec![record("src/lib.rs", 100, 100, 85)],
    )
    .expect_err("exactly 85 percent regions must fail");
    assert!(error.to_string().contains("regions"), "{error:#}");
}

#[test]
fn test_functions_equal_threshold_pass() {
    let project = project();
    evaluate(
        project.path(),
        json!({}),
        vec![record("src/lib.rs", 91, 95, 86)],
    )
    .expect("functions threshold is inclusive");
}

#[test]
fn test_empty_thresholds_keep_defaults() {
    let project = project();
    let error = evaluate(
        project.path(),
        json!({"thresholds":{}}),
        vec![record("src/lib.rs", 0, 0, 0)],
    )
    .expect_err("empty thresholds cannot disable checks");
    assert!(
        error.to_string().contains("coverage thresholds failed"),
        "{error:#}"
    );
}

#[test]
fn test_null_required_threshold_is_rejected() {
    let project = project();
    for metric in ["lines", "functions", "regions"] {
        let path = project.path().join("config.json");
        fs::write(&path, json!({"thresholds":{metric:null}}).to_string()).expect("config");
        let error = load_config(&path).expect_err("required thresholds cannot be disabled");
        assert!(error.to_string().contains(metric), "{error:#}");
    }
}

#[test]
fn test_every_source_root_requires_report_hits() {
    let project = project();
    let error = evaluate(
        project.path(),
        json!({"source_dirs":{"demo":["src", "src/extra"]}}),
        vec![record("src/lib.rs", 100, 100, 100)],
    )
    .expect_err("extra source root was not measured");
    assert!(error.to_string().contains("src/extra"), "{error:#}");
    assert!(
        report(
            project.path(),
            &project.path().join("coverage.json"),
            &project.path().join("report.json")
        )
        .is_err()
    );
}

#[test]
fn test_unknown_packages_are_rejected() {
    let project = project();
    for config in [
        json!({"source_dirs":{"unknown":["src"]}}),
        json!({"threshold_exempt_files":{"unknown":["src/lib.rs"]}}),
        json!({"exclude_packages":["unknown"]}),
    ] {
        let error = evaluate(
            project.path(),
            config,
            vec![record("src/lib.rs", 100, 100, 100)],
        )
        .expect_err("unknown package must fail");
        assert!(error.to_string().contains("unknown package"), "{error:#}");
    }
}

#[test]
fn test_missing_configured_paths_are_rejected() {
    let project = project();
    for config in [
        json!({"source_dirs":{"demo":["src","missing"]}}),
        json!({"threshold_exempt_files":{"demo":["missing.rs"]}}),
    ] {
        let error = evaluate(
            project.path(),
            config,
            vec![record("src/lib.rs", 100, 100, 100)],
        )
        .expect_err("missing path must fail");
        assert!(error.to_string().contains("does not exist"), "{error:#}");
    }
}

#[test]
fn test_empty_and_duplicate_path_lists_are_rejected() {
    let project = project();
    for config in [
        json!({"source_dirs":{"demo":[]}}),
        json!({"source_dirs":{"demo":["src","src"]}}),
        json!({"threshold_exempt_files":{"demo":["src/lib.rs","src/lib.rs"]}}),
    ] {
        assert!(
            evaluate(
                project.path(),
                config,
                vec![record("src/lib.rs", 100, 100, 100)]
            )
            .is_err()
        );
    }
}

#[test]
fn test_workspace_exemptions_only_match_the_named_package() {
    let project = project();
    package(&project.path().join("member"), "member");
    let manifest = project.path().join("Cargo.toml");
    let mut text = fs::read_to_string(&manifest).expect("manifest");
    text.push_str("\n[workspace]\nmembers = [\"member\"]\nresolver = \"3\"\n");
    fs::write(manifest, text).expect("workspace manifest");
    let files = vec![
        record("src/lib.rs", 100, 100, 100),
        record("member/src/lib.rs", 0, 0, 0),
    ];
    let config = json!({"scope":"workspace","threshold_exempt_files":{"demo":["src/lib.rs"]}});
    let error = evaluate(project.path(), config, files)
        .expect_err("member's file must not be exempted by a suffix match");
    assert!(
        error.to_string().contains("coverage thresholds failed"),
        "{error:#}"
    );
}

#[test]
fn test_workspace_source_paths_are_package_relative() {
    let project = project();
    package(&project.path().join("member"), "member");
    let manifest = project.path().join("Cargo.toml");
    let mut text = fs::read_to_string(&manifest).expect("manifest");
    text.push_str(
        "\n[workspace]\nmembers = [\"member\"]\ndefault-members = [\"member\"]\nresolver = \"3\"\n",
    );
    fs::write(manifest, text).expect("workspace manifest");
    evaluate(
        project.path(),
        json!({"source_dirs":{"member":["src"]}}),
        vec![record("member/src/lib.rs", 100, 100, 100)],
    )
    .expect("member source root must resolve");
    let error = evaluate(
        project.path(),
        json!({"source_dirs":{"demo":["src"]}}),
        vec![record("src/lib.rs", 100, 100, 100)],
    )
    .expect_err("non-default package must be rejected");
    assert!(
        error.to_string().contains("unselected package"),
        "{error:#}"
    );
}

#[test]
fn test_virtual_workspace_roots_and_exclusions() {
    let directory = tempfile::tempdir().expect("workspace");
    let root = directory.path();
    package(&root.join("first"), "first");
    package(&root.join("second"), "second");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"first\", \"second\"]\nresolver = \"3\"\n",
    )
    .expect("virtual manifest");
    let config = json!({"scope":"workspace","exclude_packages":["second"]});
    evaluate(
        root,
        config,
        vec![record("first/src/lib.rs", 100, 100, 100)],
    )
    .expect("excluded package requires no report hits");
    let error = evaluate(
        root,
        json!({"scope":"package"}),
        vec![record("first/src/lib.rs", 100, 100, 100)],
    )
    .expect_err("virtual workspace has no root package");
    assert!(
        error.to_string().contains("scope contains no packages"),
        "{error:#}"
    );
    let error = evaluate(
        root,
        json!({"scope":"workspace","exclude_packages":["second"],
        "threshold_exempt_files":{"second":["src/lib.rs"]}}),
        vec![record("first/src/lib.rs", 100, 100, 100)],
    )
    .expect_err("excluded package cannot configure exemptions");
    assert!(
        error.to_string().contains("unselected package 'second'"),
        "{error:#}"
    );
}

#[test]
fn test_exempt_report_hits_still_satisfy_source_root_check() {
    let project = project();
    let root = project.path();
    let absolute = root.join("src/extra/item.rs");
    evaluate(
        root,
        json!({"source_dirs":{"demo":["src","src/extra"]},
        "threshold_exempt_files":{"demo":["src/extra/item.rs"]}}),
        vec![
            record("src/lib.rs", 100, 100, 100),
            record(absolute.to_str().expect("UTF-8 path"), 0, 0, 0),
        ],
    )
    .expect("exempt files prove root coverage but do not lower aggregates");
}

#[test]
fn test_source_roots_do_not_match_sibling_prefixes() {
    let project = project();
    let root = project.path();
    fs::create_dir_all(root.join("src/extra_suffix")).expect("sibling directory");
    fs::write(root.join("src/extra_suffix/item.rs"), "").expect("sibling source");
    let error = evaluate(
        root,
        json!({"source_dirs":{"demo":["src/extra"]}}),
        vec![record("src/extra_suffix/item.rs", 100, 100, 100)],
    )
    .expect_err("directory names require component boundaries");
    assert!(
        error.to_string().contains("matched no coverage files"),
        "{error:#}"
    );
}

#[test]
fn test_invalid_path_kinds_and_empty_scope_fail() {
    let project = project();
    for (config, message) in [
        (
            json!({"source_dirs":{"demo":["src/lib.rs"]}}),
            "source directory does not exist",
        ),
        (
            json!({"threshold_exempt_files":{"demo":["src"]}}),
            "exemption file does not exist",
        ),
        (
            json!({"exclude_packages":["demo"]}),
            "scope contains no packages",
        ),
        (
            json!({"exclude_packages":["demo","demo"]}),
            "must not contain duplicates",
        ),
    ] {
        let error = evaluate(
            project.path(),
            config,
            vec![record("src/lib.rs", 100, 100, 100)],
        )
        .expect_err("invalid policy must fail");
        assert!(error.to_string().contains(message), "{error:#}");
    }
}

#[test]
fn test_partial_thresholds_keep_other_required_defaults() {
    let project = project();
    let path = project.path().join("config.json");
    fs::write(&path, r#"{"thresholds":{"lines":92}}"#).expect("config");
    let config = load_config(&path).expect("partial policy");
    assert_eq!(config.thresholds.lines, Some(92.0));
    assert_eq!(config.thresholds.functions, Some(95.0));
    assert_eq!(config.thresholds.regions, Some(85.0));
}

#[test]
fn test_cli_reports_policy_failures_with_nonzero_exit() {
    let project = project();
    let root = project.path();
    let _ = evaluate(root, json!({}), vec![record("src/lib.rs", 90, 100, 100)]);
    let output = Command::new(env!("CARGO_BIN_EXE_rs-infra-coverage"))
        .arg("--project")
        .arg(root)
        .arg("--config")
        .arg(root.join("coverage.json"))
        .args(["check", "--input"])
        .arg(root.join("report.json"))
        .output()
        .expect("coverage CLI");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lines <= 90.00"), "{stderr}");
}
