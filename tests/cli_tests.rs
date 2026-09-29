//! Regression tests for CLI completion summaries.

use std::fs;
use std::process::Command;

use serde_json::json;
use tempfile::TempDir;

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rs-infra-coverage"))
        .args(arguments)
        .output()
        .expect("run coverage CLI")
}

fn project() -> TempDir {
    let directory = tempfile::tempdir().expect("temporary project");
    fs::create_dir_all(directory.path().join(".infra/ci")).expect("configuration directory");
    fs::create_dir_all(directory.path().join("src")).expect("source directory");
    fs::write(
        directory.path().join("Cargo.toml"),
        "[package]\nname = \"cli-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::write(directory.path().join("src/lib.rs"), "").expect("source file");
    fs::write(directory.path().join(".infra/ci/coverage.json"), "{}").expect("configuration");
    fs::write(
        directory.path().join("coverage.json"),
        json!({"data":[{"files":[{"filename":"src/lib.rs","summary":{
            "lines":{"covered":95,"count":100,"percent":95.0},
            "functions":{"covered":95,"count":100,"percent":95.0},
            "regions":{"covered":95,"count":100,"percent":95.0}
        }}]}]})
        .to_string(),
    )
    .expect("coverage report");
    directory
}

#[test]
fn cli_summaries_prefix_success_and_failure_with_status_icons() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("✅ rs-infra-coverage: help succeeded"));

    let parse_error = run(&["--invalid-option"]);
    assert!(!parse_error.status.success());
    assert!(String::from_utf8_lossy(&parse_error.stderr).contains("❌ rs-infra-coverage: failed (exit code"));

    let project = project();
    let project_path = project.path().to_str().expect("UTF-8 project path");
    let report_path = project.path().join("coverage.json");
    let report_path = report_path.to_str().expect("UTF-8 report path");
    let success = run(&["--project", project_path, "report", "--input", report_path]);
    assert!(success.status.success(), "{}", String::from_utf8_lossy(&success.stderr));
    assert!(String::from_utf8_lossy(&success.stdout).contains("✅ rs-infra-coverage: report succeeded"));

    let missing_path = project.path().join("missing.json");
    let missing_path = missing_path.to_str().expect("UTF-8 missing report path");
    let failure = run(&["--project", project_path, "report", "--input", missing_path]);
    assert!(!failure.status.success());
    assert!(String::from_utf8_lossy(&failure.stderr).contains("❌ rs-infra-coverage: failed:"));
}
