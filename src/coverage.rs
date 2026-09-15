// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Coverage data loading, filtering, collection, and reporting operations.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use serde_json::Value;
use serde_json::from_str;

use crate::Config;
use crate::Thresholds;
use crate::coverage_plan::CoveragePlan;
use crate::file_coverage::FileCoverage;

/// Loads and validates a coverage configuration file.
///
/// A missing file produces the default configuration. Reading and parsing the
/// file performs filesystem IO and returns contextual errors for invalid JSON
/// or invalid policy values.
///
/// # Parameters
///
/// * `path` - The configuration file to read.
///
/// # Returns
///
/// The parsed configuration, or the default configuration when `path` is
/// missing.
///
/// # Errors
///
/// Returns an error when the file cannot be read, contains invalid JSON, or
/// violates a supported configuration constraint.
pub fn load_config(path: &Path) -> Result<Config> {
    if !path.is_file() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(path)?;
    let config: Config = from_str(&text).context("invalid coverage configuration")?;
    validate_config(&config)?;
    Ok(config)
}

/// Resolves the configured coverage file, including the legacy fallback.
///
/// If `configured` is missing and is the modern project path, an existing
/// `.rs-ci-coverage.json` is selected and a migration warning is printed.
/// This function does not read or modify either configuration file.
///
/// # Parameters
///
/// * `project` - The project root used to locate the fallback.
/// * `configured` - The configured path, absolute or project-relative.
///
/// # Returns
///
/// The existing configured path or the legacy fallback path.
///
/// # Errors
///
/// This function currently returns no operational errors; the result type is
/// kept consistent with the public command API.
pub fn resolve_config_path(project: &Path, configured: &Path) -> Result<PathBuf> {
    if configured.is_file() {
        return Ok(configured.to_path_buf());
    }
    let modern = project.join(".infra/ci/coverage.json");
    let legacy = project.join(".rs-ci-coverage.json");
    if configured == modern && legacy.is_file() {
        eprintln!(
            "warning: using legacy coverage configuration {}; migrate to {}",
            legacy.display(),
            modern.display()
        );
        return Ok(legacy);
    }
    Ok(configured.to_path_buf())
}

/// Collects LLVM coverage data and checks the resulting report.
///
/// This function creates the output parent directory, starts `cargo
/// llvm-cov`, writes the report, and then applies the configured thresholds.
///
/// # Parameters
///
/// * `project` - The project root in which Cargo is executed.
/// * `config_path` - The coverage configuration file.
/// * `output` - Optional report path; defaults to `target/infra/coverage/raw.json`.
///
/// # Errors
///
/// Returns an error when configuration loading, directory creation, process
/// startup, Cargo metadata/path validation, collection, report parsing, or
/// threshold checking fails.
pub fn collect(project: &Path, config_path: &Path, output: Option<&Path>) -> Result<()> {
    let config = load_config(config_path)?;
    let default_output = project.join("target/infra/coverage/raw.json");
    let output = output.unwrap_or(&default_output);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut command = Command::new("cargo");
    command.args(collection_args(&config, project)?);
    let status = command
        .arg(output)
        .current_dir(project)
        .status()
        .context("failed to start cargo llvm-cov")?;
    if !status.success() {
        bail!("cargo llvm-cov failed");
    }
    check(project, config_path, output)
}

/// Runs Clippy with the configured optional coverage cfg flag.
///
/// This function starts `cargo clippy` in `project`. When enabled by the
/// argument, configuration, or environment, it sets `RUSTFLAGS` for that
/// child process only.
///
/// # Parameters
///
/// * `project` - The project root in which Clippy is executed.
/// * `config_path` - The coverage configuration file.
/// * `coverage_cfg` - Explicitly enables `--cfg coverage` when true.
///
/// # Errors
///
/// Returns an error when configuration loading, process startup, or Clippy
/// execution fails.
pub fn clippy(project: &Path, config_path: &Path, coverage_cfg: bool) -> Result<()> {
    let config = load_config(config_path)?;
    let use_coverage_cfg = coverage_cfg
        || config.clippy.coverage_cfg
        || config.coverage_cfg_clippy
        || env::var("RUN_COVERAGE_CFG_CLIPPY").as_deref() == Ok("1");
    let mut command = Command::new("cargo");
    command.args([
        "clippy",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ]);
    if use_coverage_cfg {
        command.env("RUSTFLAGS", "--cfg coverage");
    }
    let status = command
        .current_dir(project)
        .status()
        .context("failed to start cargo clippy")?;
    if !status.success() {
        bail!("cargo clippy failed");
    }
    Ok(())
}

/// Builds the stable argument list for `cargo llvm-cov`.
///
/// The returned arguments omit the output path, which is appended by
/// [`collect`].
///
/// # Errors
///
/// Returns an error when Cargo metadata, package selection, or configured
/// filesystem paths cannot be validated.
///
/// # Parameters
///
/// * `config` - The validated coverage configuration.
/// * `project` - The project root containing the Cargo manifest.
fn collection_args(config: &Config, project: &Path) -> Result<Vec<String>> {
    let plan = CoveragePlan::load(project, config)?;
    let mut args = vec!["llvm-cov".into()];
    args.extend(plan.cargo_args);
    args.extend([
        "--all-features".into(),
        "--json".into(),
        "--output-path".into(),
    ]);
    Ok(args)
}

/// Checks the selected files against configured coverage thresholds.
///
/// The input report is read from disk; files outside configured source roots
/// or listed as exemptions are ignored. Selected metric hit counts are summed
/// before comparison, so larger source files contribute proportionally more.
///
/// # Errors
///
/// Returns an error when configuration or report parsing fails, no files are
/// selected, Cargo metadata or configured paths are invalid, a source root has
/// no report files, or a threshold comparison fails. Lines and regions require
/// strictly greater coverage; functions and branches allow equality.
///
/// # Parameters
///
/// * `project` - The project root used to interpret source directories.
/// * `config_path` - The coverage configuration file to load.
/// * `input` - The LLVM coverage JSON report to read.
pub fn check(project: &Path, config_path: &Path, input: &Path) -> Result<()> {
    let config = load_config(config_path)?;
    let selected = select_files(project, &config, read_files(input)?)?;
    if selected.is_empty() {
        bail!("coverage report contains no files selected by the configured source roots");
    }
    report_files(
        &selected
            .iter()
            .map(|file| file.coverage.clone())
            .collect::<Vec<_>>(),
    );
    let failures = threshold_failures(&config.thresholds, &selected);
    if !failures.is_empty() {
        bail!("coverage thresholds failed: {}", failures.join(", "));
    }
    Ok(())
}

/// Reports selected coverage files without applying thresholds.
///
/// The input report is read from disk and filtered using the configured source
/// roots and exemptions before being printed to standard output.
///
/// # Errors
///
/// Returns an error when configuration or report parsing fails, or when no
/// files are selected, Cargo metadata or configured paths are invalid, or any
/// source root has no report files. The source checkout must be available.
///
/// # Parameters
///
/// * `project` - The project root used to interpret source directories.
/// * `config_path` - The coverage configuration file to load.
/// * `input` - The LLVM coverage JSON report to read.
pub fn report(project: &Path, config_path: &Path, input: &Path) -> Result<()> {
    let config = load_config(config_path)?;
    let selected = select_files(project, &config, read_files(input)?)?;
    if selected.is_empty() {
        bail!("coverage report contains no files selected by the configured source roots");
    }
    report_files(
        &selected
            .iter()
            .map(|file| file.coverage.clone())
            .collect::<Vec<_>>(),
    );
    Ok(())
}

/// Validates all configured scope, threshold, and path constraints.
///
/// # Errors
///
/// Returns an error identifying the first unsupported scope, invalid threshold,
/// empty package name, or unsafe path encountered.
///
/// # Parameters
///
/// * `config` - The configuration values to validate before execution.
fn validate_config(config: &Config) -> Result<()> {
    if config.exclude_packages.iter().any(|name| name.is_empty()) {
        bail!("exclude_packages must contain non-empty strings");
    }
    if let Some(scope) = config.scope.as_deref()
        && !matches!(scope, "default-members" | "workspace" | "package")
    {
        bail!("scope must be one of default-members, workspace, or package");
    }
    for (name, threshold) in [
        ("lines", config.thresholds.lines),
        ("functions", config.thresholds.functions),
        ("regions", config.thresholds.regions),
    ] {
        if threshold.is_none() {
            bail!("{name} coverage threshold cannot be disabled");
        }
    }
    if has_duplicates(&config.exclude_packages) {
        bail!("exclude_packages must not contain duplicates");
    }
    if config.source_dirs.values().any(Vec::is_empty) {
        bail!("each source_dirs array must be non-empty");
    }
    for threshold in [
        config.thresholds.lines,
        config.thresholds.functions,
        config.thresholds.regions,
        config.thresholds.branches,
    ]
    .into_iter()
    .flatten()
    {
        if !(0.0..=100.0).contains(&threshold) {
            bail!("coverage thresholds must be between 0 and 100");
        }
    }
    for (package, paths) in config
        .source_dirs
        .iter()
        .chain(config.threshold_exempt_files.iter())
    {
        if has_duplicates(paths) {
            bail!("coverage paths for package '{package}' must not contain duplicates");
        }
        if package.is_empty() || paths.iter().any(|path| !valid_relative_path(path)) {
            bail!("coverage paths must be non-empty relative paths without '..'");
        }
    }
    Ok(())
}

/// Returns whether a configuration list repeats a value, without modifying it.
fn has_duplicates(values: &[String]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

/// Reports whether a path is a safe, non-empty relative path.
///
/// # Parameters
///
/// * `path` - The configuration path to validate.
///
/// # Returns
///
/// `true` when `path` is relative, non-empty, free of parent traversal, and
/// contains no control characters.
fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && !path.split('/').any(|part| part == "..")
        && !path.chars().any(|ch| ch.is_control())
}

/// Stores the covered and total item counts for one coverage metric.
#[derive(Debug, Clone, Copy)]
struct MetricCounts {
    /// Number of covered items reported by LLVM.
    covered: u64,
    /// Total number of items measured by LLVM.
    count: u64,
}

/// Retains LLVM hit counts for weighted aggregate threshold evaluation.
#[derive(Debug)]
struct CoverageRecord {
    /// Display metrics and source path for this file.
    coverage: FileCoverage,
    /// Line counts reported for this file.
    lines: Option<MetricCounts>,
    /// Function counts reported for this file.
    functions: Option<MetricCounts>,
    /// Region counts reported for this file.
    regions: Option<MetricCounts>,
    /// Branch counts reported for this file.
    branches: Option<MetricCounts>,
}

/// Selects one metric's counts from a coverage record.
type MetricCountsGetter = fn(&CoverageRecord) -> Option<MetricCounts>;
type MetricPercentGetter = fn(&CoverageRecord) -> Option<f64>;

/// Reads covered and total counts from one LLVM metric object.
fn metric_counts(value: Option<&Value>) -> Option<MetricCounts> {
    let value = value?;
    Some(MetricCounts {
        covered: value.get("covered")?.as_u64()?,
        count: value.get("count")?.as_u64()?,
    })
}

/// Reads file summaries and hit counts from an LLVM coverage JSON report.
///
/// # Errors
///
/// Returns an error when the file cannot be read, the JSON is malformed, the
/// `data` array is absent, or a file record has no filename.
///
/// # Parameters
///
/// * `path` - The LLVM coverage JSON report to read.
fn read_files(path: &Path) -> Result<Vec<CoverageRecord>> {
    let value: Value = from_str(&fs::read_to_string(path)?)?;
    let records = value
        .get("data")
        .and_then(Value::as_array)
        .context("coverage JSON must contain a data array")?;
    let mut files = Vec::new();
    for record in records {
        for file in record
            .get("files")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let filename = file
                .get("filename")
                .and_then(Value::as_str)
                .context("coverage file has no filename")?
                .to_owned();
            let summary = file.get("summary");
            let lines = summary.and_then(|summary| summary.get("lines"));
            let functions = summary.and_then(|summary| summary.get("functions"));
            let regions = summary.and_then(|summary| summary.get("regions"));
            let branches = summary.and_then(|summary| summary.get("branches"));
            files.push(CoverageRecord {
                coverage: FileCoverage {
                    filename,
                    lines_percent: percent(lines),
                    functions_percent: percent(functions),
                    regions_percent: percent(regions),
                    branches_percent: percent(branches),
                },
                lines: metric_counts(lines),
                functions: metric_counts(functions),
                regions: metric_counts(regions),
                branches: metric_counts(branches),
            });
        }
    }
    Ok(files)
}

/// Extracts a percentage value from an LLVM metric object.
///
/// # Parameters
///
/// * `value` - An optional JSON metric object containing a numeric `percent`
///   field.
///
/// # Returns
///
/// The reported percentage, or `None` when the object or field is absent or
/// non-numeric.
fn percent(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(|value| value.get("percent"))
        .and_then(Value::as_f64)
}

/// Resolves report paths and validates that every configured root was measured.
///
/// Runs Cargo metadata and checks configured package paths through the plan.
/// Root matching precedes exemptions, so exempt files still prove their source
/// root was measured. Returns an error for an invalid plan or an unmatched root.
fn select_files(
    project: &Path,
    config: &Config,
    files: Vec<CoverageRecord>,
) -> Result<Vec<CoverageRecord>> {
    let plan = CoveragePlan::load(project, config)?;
    let paths: Vec<_> = files
        .iter()
        .map(|file| plan.report_path(&file.coverage.filename))
        .collect();
    for root in &plan.roots {
        if !paths
            .iter()
            .any(|path| path.starts_with(root) && path != root)
        {
            bail!("source root '{}' matched no coverage files", root.display());
        }
    }
    Ok(files
        .into_iter()
        .zip(paths)
        .filter(|(_, path)| {
            plan.roots
                .iter()
                .any(|root| path.starts_with(root) && path != root)
                && !plan.exemptions.contains(path)
        })
        .map(|(file, _)| file)
        .collect())
}

/// Computes threshold failures from the selected file metrics.
///
/// Each configured metric is computed from the combined covered and total
/// counts across selected files. A configured metric with no counts is
/// reported as a failure.
///
/// # Parameters
///
/// * `thresholds` - The minimum percentages to enforce.
/// * `files` - The files selected for threshold evaluation.
///
/// # Returns
///
/// A list of human-readable failures, empty when every configured threshold
/// is satisfied.
fn threshold_failures(thresholds: &Thresholds, files: &[CoverageRecord]) -> Vec<String> {
    let metrics: [(&str, Option<f64>, MetricCountsGetter, MetricPercentGetter); 4] = [
        (
            "lines",
            thresholds.lines,
            |file| file.lines,
            |file| file.coverage.lines_percent,
        ),
        (
            "functions",
            thresholds.functions,
            |file| file.functions,
            |file| file.coverage.functions_percent,
        ),
        (
            "regions",
            thresholds.regions,
            |file| file.regions,
            |file| file.coverage.regions_percent,
        ),
        (
            "branches",
            thresholds.branches,
            |file| file.branches,
            |file| file.coverage.branches_percent,
        ),
    ];
    let mut failures = Vec::new();
    for (name, threshold, counts, percentage) in metrics {
        let missing_counts = files
            .iter()
            .any(|file| percentage(file).is_some() && counts(file).is_none());
        let (covered, count) = files
            .iter()
            .filter_map(counts)
            .fold((0_u64, 0_u64), |(covered, count), metric| {
                (covered + metric.covered, count + metric.count)
            });
        let percent = (count > 0).then(|| covered as f64 / count as f64 * 100.0);
        if let Some(threshold) = threshold {
            if missing_counts {
                failures.push(format!("{name} counts unavailable"));
            } else if let Some(percent) = percent.filter(|percent| {
                if matches!(name, "lines" | "regions") {
                    *percent <= threshold
                } else {
                    *percent < threshold
                }
            }) {
                let operator = if matches!(name, "lines" | "regions") {
                    "<="
                } else {
                    "<"
                };
                failures.push(format!("{name} {operator} {threshold:.2} ({percent:.2}%)"));
            } else if percent.is_none() {
                failures.push(format!("{name} counts unavailable"));
            }
        }
    }
    failures
}

/// Prints the selected file count and metric summaries to standard output.
///
/// # Parameters
///
/// * `files` - The selected coverage files whose metrics should be printed.
fn report_files(files: &[FileCoverage]) {
    println!("Coverage files: {}", files.len());
    for file in files {
        println!(
            "  {} lines={:?} functions={:?} regions={:?} branches={:?}",
            file.filename,
            file.lines_percent,
            file.functions_percent,
            file.regions_percent,
            file.branches_percent
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Config;
    use super::check;
    use super::collection_args;
    use super::load_config;
    use super::read_files;
    use super::resolve_config_path;
    use super::validate_config;

    #[test]
    fn rejects_invalid_source_path() {
        let config = Config {
            source_dirs: [("pkg".into(), vec!["../src".into()])]
                .into_iter()
                .collect(),
            ..Config::default()
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn applies_thresholds_to_weighted_coverage_counts() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path();
        fs::create_dir_all(project.join("src")).expect("source root");
        fs::write(project.join("src/lib.rs"), "").expect("library target");
        fs::write(
            project.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("package manifest");
        let config = project.join("coverage.json");
        let input = project.join("report.json");
        fs::write(&config, r#"{"thresholds":{"lines":92}}"#).unwrap();
        fs::write(
            &input,
            r#"{"data":[{"files":[
                {"filename":"src/small.rs","summary":{"lines":{"covered":1,"count":1,"percent":100.0}}},
                {"filename":"src/large.rs","summary":{"lines":{"covered":8,"count":9,"percent":88.8888888889}}}
            ]}]}"#,
        )
        .unwrap();

        let error = check(project, &config, &input).unwrap_err().to_string();
        assert!(error.contains("lines <= 92.00 (90.00%)"));
    }

    #[test]
    fn rejects_partial_metric_counts_during_threshold_checks() {
        let directory = tempfile::tempdir().unwrap();
        let project = directory.path();
        fs::create_dir_all(project.join("src")).expect("source root");
        fs::write(project.join("src/lib.rs"), "").expect("library target");
        fs::write(
            project.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .expect("package manifest");
        let config = project.join("coverage.json");
        let input = project.join("report.json");
        fs::write(&config, r#"{"thresholds":{"lines":0}}"#).unwrap();
        fs::write(
            &input,
            r#"{"data":[{"files":[
                {"filename":"src/counts.rs","summary":{"lines":{"covered":1,"count":1,"percent":100.0}}},
                {"filename":"src/percent-only.rs","summary":{"lines":{"percent":100.0}}}
            ]}]}"#,
        )
        .unwrap();

        let error = check(project, &config, &input).expect_err("partial counts must fail");
        assert!(
            error.to_string().contains("lines counts unavailable"),
            "{error:#}"
        );
    }

    #[test]
    fn reads_llvm_file_summary() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("coverage.json");
        fs::write(
            &input,
            r#"{"data":[{"files":[{"filename":"src/lib.rs","summary":{"lines":{"percent":91.0}}}]}]}"#,
        )
        .unwrap();
        assert_eq!(
            read_files(&input).unwrap()[0].coverage.lines_percent,
            Some(91.0)
        );
    }

    #[test]
    fn parses_legacy_scope_threshold_and_clippy_settings() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("coverage.json");
        fs::write(
            &input,
            r#"{
                "scope":"package",
                "exclude_packages":["support"],
                "thresholds":{"lines":91,"functions":92,"regions":93,"branches":94},
                "clippy":{"coverage_cfg":true}
            }"#,
        )
        .unwrap();
        let config = load_config(&input).unwrap();
        assert_eq!(config.scope.as_deref(), Some("package"));
        assert_eq!(config.thresholds.lines, Some(91.0));
        assert!(config.clippy.coverage_cfg);
    }

    #[test]
    fn parses_top_level_legacy_clippy_setting() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("coverage.json");
        fs::write(&input, r#"{"run_coverage_cfg_clippy":true}"#).unwrap();
        assert!(load_config(&input).unwrap().coverage_cfg_clippy);
    }

    #[test]
    fn rejects_invalid_scope_and_threshold() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("coverage.json");
        fs::write(&input, r#"{"scope":"all","thresholds":{"lines":101}}"#).unwrap();
        assert!(load_config(&input).is_err());
    }

    #[test]
    fn builds_stable_collection_arguments() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        fs::create_dir_all(directory.path().join("src")).expect("source root");
        fs::write(directory.path().join("src/lib.rs"), "").expect("library target");
        let config = Config {
            scope: Some("package".into()),
            ..Config::default()
        };
        assert_eq!(
            collection_args(&config, directory.path()).unwrap(),
            vec![
                "llvm-cov",
                "--package",
                "demo",
                "--all-features",
                "--json",
                "--output-path"
            ]
        );
    }

    #[test]
    fn resolves_legacy_config_when_modern_config_is_missing() {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".infra/ci")).unwrap();
        let legacy = directory.path().join(".rs-ci-coverage.json");
        fs::write(&legacy, "{}").unwrap();
        let modern = directory.path().join(".infra/ci/coverage.json");
        assert_eq!(
            resolve_config_path(directory.path(), &modern).unwrap(),
            legacy
        );
    }
}
