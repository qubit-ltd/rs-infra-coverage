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
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    pub lines: Option<f64>,
    pub functions: Option<f64>,
    pub regions: Option<f64>,
    pub branches: Option<f64>,
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

pub fn collect(project: &Path, config_path: &Path, output: Option<&Path>) -> Result<()> {
    let _ = load_config(config_path)?;
    let default_output = project.join("target/infra/coverage/raw.json");
    let output = output.unwrap_or(&default_output);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let status = Command::new("cargo")
        .args([
            "llvm-cov",
            "--workspace",
            "--all-features",
            "--json",
            "--output-path",
        ])
        .arg(output)
        .current_dir(project)
        .status()
        .context("failed to start cargo llvm-cov")?;
    if !status.success() {
        bail!("cargo llvm-cov failed");
    }
    check(project, config_path, output)
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
}
