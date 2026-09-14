//! Coverage collection, validation, and reporting.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub scope: Option<String>,
    #[serde(default)]
    pub exclude_packages: Vec<String>,
    #[serde(default)]
    pub source_dirs: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub threshold_exempt_files: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub thresholds: Thresholds,
    #[serde(default)]
    pub clippy: ClippyConfig,
    #[serde(default, alias = "run_coverage_cfg_clippy")]
    pub coverage_cfg_clippy: bool,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClippyConfig {
    #[serde(alias = "run_coverage_cfg", alias = "run_coverage_cfg_clippy")]
    pub coverage_cfg: bool,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub lines: Option<f64>,
    pub functions: Option<f64>,
    pub regions: Option<f64>,
    pub branches: Option<f64>,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            lines: Some(90.0),
            functions: Some(95.0),
            regions: Some(85.0),
            branches: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileCoverage {
    pub filename: String,
    pub lines_percent: Option<f64>,
    pub functions_percent: Option<f64>,
    pub regions_percent: Option<f64>,
    pub branches_percent: Option<f64>,
}

pub fn load_config(path: &Path) -> Result<Config> {
    if !path.is_file() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&text).context("invalid coverage configuration")?;
    validate_config(&config)?;
    Ok(config)
}

pub fn resolve_config_path(project: &Path, configured: &Path) -> Result<std::path::PathBuf> {
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

pub fn clippy(project: &Path, config_path: &Path, coverage_cfg: bool) -> Result<()> {
    let config = load_config(config_path)?;
    let use_coverage_cfg = coverage_cfg
        || config.clippy.coverage_cfg
        || config.coverage_cfg_clippy
        || std::env::var("RUN_COVERAGE_CFG_CLIPPY").as_deref() == Ok("1");
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

fn collection_args(config: &Config, project: &Path) -> Result<Vec<String>> {
    let mut args = vec!["llvm-cov".into()];
    match config.scope.as_deref().unwrap_or("default-members") {
        "default-members" => {}
        "workspace" => args.push("--workspace".into()),
        "package" => {
            args.push("--package".into());
            args.push(package_name(project)?);
        }
        scope => bail!("scope must be one of default-members, workspace, or package; got {scope}"),
    }
    args.push("--all-features".into());
    for package in &config.exclude_packages {
        args.push("--exclude".into());
        args.push(package.clone());
    }
    args.extend(["--json".into(), "--output-path".into()]);
    Ok(args)
}

fn package_name(project: &Path) -> Result<String> {
    let manifest = fs::read_to_string(project.join("Cargo.toml"))?;
    manifest
        .lines()
        .find_map(|line| {
            line.strip_prefix("name = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .map(str::to_owned)
        .context("package scope requires a package name in Cargo.toml")
}

pub fn check(project: &Path, config_path: &Path, input: &Path) -> Result<()> {
    let config = load_config(config_path)?;
    let files = read_files(input)?;
    let selected: Vec<_> = files
        .into_iter()
        .filter(|file| selected_file(project, &config, file))
        .collect();
    if selected.is_empty() {
        bail!("coverage report contains no files selected by the configured source roots");
    }
    let failures = threshold_failures(&config.thresholds, &selected);
    if !failures.is_empty() {
        bail!("coverage thresholds failed: {}", failures.join(", "));
    }
    report_files(&selected);
    Ok(())
}

pub fn report(project: &Path, config_path: &Path, input: &Path) -> Result<()> {
    let config = load_config(config_path)?;
    let files = read_files(input)?;
    let selected: Vec<_> = files
        .into_iter()
        .filter(|file| selected_file(project, &config, file))
        .collect();
    if selected.is_empty() {
        bail!("coverage report contains no files selected by the configured source roots");
    }
    report_files(&selected);
    Ok(())
}

fn validate_config(config: &Config) -> Result<()> {
    if config.exclude_packages.iter().any(|name| name.is_empty()) {
        bail!("exclude_packages must contain non-empty strings");
    }
    if let Some(scope) = config.scope.as_deref()
        && !matches!(scope, "default-members" | "workspace" | "package")
    {
        bail!("scope must be one of default-members, workspace, or package");
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
        if package.is_empty() || paths.iter().any(|path| !valid_relative_path(path)) {
            bail!("coverage paths must be non-empty relative paths without '..'");
        }
    }
    Ok(())
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.split('/').any(|part| part == "..")
        && !path.chars().any(|ch| ch.is_control())
}

fn read_files(path: &Path) -> Result<Vec<FileCoverage>> {
    let value: Value = serde_json::from_str(&fs::read_to_string(path)?)?;
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
            files.push(FileCoverage {
                filename,
                lines_percent: percent(
                    file.get("summary").and_then(|summary| summary.get("lines")),
                ),
                functions_percent: percent(
                    file.get("summary")
                        .and_then(|summary| summary.get("functions")),
                ),
                regions_percent: percent(
                    file.get("summary")
                        .and_then(|summary| summary.get("regions")),
                ),
                branches_percent: percent(
                    file.get("summary")
                        .and_then(|summary| summary.get("branches")),
                ),
            });
        }
    }
    Ok(files)
}

fn percent(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(|value| value.get("percent"))
        .and_then(Value::as_f64)
}

fn selected_file(project: &Path, config: &Config, file: &FileCoverage) -> bool {
    let path = Path::new(&file.filename);
    let relative = path.strip_prefix(project).unwrap_or(path).to_string_lossy();
    if config
        .threshold_exempt_files
        .values()
        .flatten()
        .any(|excluded| relative.ends_with(excluded))
    {
        return false;
    }
    if config.source_dirs.is_empty() {
        return relative.starts_with("src/") || relative == "src";
    }
    config.source_dirs.values().flatten().any(|source| {
        relative.starts_with(&format!("{source}/")) || relative.as_ref() == source.as_str()
    })
}

fn threshold_failures(thresholds: &Thresholds, files: &[FileCoverage]) -> Vec<String> {
    let mut failures = Vec::new();
    for (name, threshold, values) in [
        (
            "lines",
            thresholds.lines,
            files
                .iter()
                .filter_map(|file| file.lines_percent)
                .collect::<Vec<_>>(),
        ),
        (
            "functions",
            thresholds.functions,
            files
                .iter()
                .filter_map(|file| file.functions_percent)
                .collect(),
        ),
        (
            "regions",
            thresholds.regions,
            files
                .iter()
                .filter_map(|file| file.regions_percent)
                .collect(),
        ),
        (
            "branches",
            thresholds.branches,
            files
                .iter()
                .filter_map(|file| file.branches_percent)
                .collect(),
        ),
    ] {
        if let Some(threshold) = threshold
            && (values.is_empty() || values.iter().sum::<f64>() / (values.len() as f64) < threshold)
        {
            failures.push(format!("{name} < {threshold:.2}"));
        }
    }
    failures
}

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
    use super::*;

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
    fn reads_llvm_file_summary() {
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("coverage.json");
        fs::write(&input, r#"{"data":[{"files":[{"filename":"src/lib.rs","summary":{"lines":{"percent":91.0}}}]}]}"#).unwrap();
        assert_eq!(read_files(&input).unwrap()[0].lines_percent, Some(91.0));
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
        let config = Config {
            scope: Some("package".into()),
            exclude_packages: vec!["support".into()],
            ..Config::default()
        };
        assert_eq!(
            collection_args(&config, directory.path()).unwrap(),
            vec![
                "llvm-cov",
                "--package",
                "demo",
                "--all-features",
                "--exclude",
                "support",
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
