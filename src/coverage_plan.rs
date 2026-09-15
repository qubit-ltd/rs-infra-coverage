// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolves coverage scope and configured paths against Cargo workspace metadata.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use serde_json::Value;

use crate::Config;

/// Validated package selection and absolute source/exemption paths.
pub(crate) struct CoveragePlan {
    /// Cargo flags selecting exactly the packages whose roots are checked.
    pub(crate) cargo_args: Vec<String>,
    /// Existing, canonical source directories; every root must match a report.
    pub(crate) roots: Vec<PathBuf>,
    /// Existing, canonical files excluded only from threshold calculations.
    pub(crate) exemptions: Vec<PathBuf>,
    /// Canonical directory used to resolve relative report filenames.
    project: PathBuf,
}

impl CoveragePlan {
    /// Loads Cargo metadata and resolves `config` for `project`.
    ///
    /// Runs `cargo metadata --no-deps` and reads configured filesystem paths.
    /// Returns an error for invalid metadata, unknown/unselected packages, an
    /// empty scope, missing paths, or duplicate canonical source directories.
    pub(crate) fn load(project: &Path, config: &Config) -> Result<Self> {
        let project = project
            .canonicalize()
            .context("cannot resolve project directory")?;
        let output = Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version", "1"])
            .arg("--manifest-path")
            .arg(project.join("Cargo.toml"))
            .current_dir(&project)
            .output()
            .context("failed to start cargo metadata")?;
        if !output.status.success() {
            bail!(
                "cargo metadata failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let metadata: Value =
            serde_json::from_slice(&output.stdout).context("invalid cargo metadata JSON")?;
        let members = metadata["workspace_members"]
            .as_array()
            .context("cargo metadata has no workspace_members")?;
        let defaults = metadata["workspace_default_members"]
            .as_array()
            .context("cargo metadata has no workspace_default_members")?;
        let packages = metadata["packages"]
            .as_array()
            .context("cargo metadata has no packages")?;
        let manifest = project.join("Cargo.toml").canonicalize()?;
        let scope = config.scope.as_deref().unwrap_or("default-members");
        let mut package_roots = BTreeMap::new();
        let mut selected = BTreeSet::new();
        for package in packages {
            if !members.contains(&package["id"]) {
                continue;
            }
            let name = package["name"].as_str().context("package has no name")?;
            let package_manifest = Path::new(
                package["manifest_path"]
                    .as_str()
                    .context("package has no manifest_path")?,
            )
            .canonicalize()?;
            let root = package_manifest
                .parent()
                .context("manifest has no parent")?;
            package_roots.insert(name.to_owned(), root.to_path_buf());
            let included = match scope {
                "workspace" => true,
                "default-members" => defaults.contains(&package["id"]),
                "package" => package_manifest == manifest,
                _ => bail!("unsupported coverage scope: {scope}"),
            };
            if included
                && !config
                    .exclude_packages
                    .iter()
                    .any(|excluded| excluded == name)
            {
                selected.insert(name.to_owned());
            }
        }
        for name in config
            .exclude_packages
            .iter()
            .chain(config.source_dirs.keys())
            .chain(config.threshold_exempt_files.keys())
        {
            if !package_roots.contains_key(name) {
                bail!("coverage configuration names unknown package '{name}'");
            }
        }
        for name in config
            .source_dirs
            .keys()
            .chain(config.threshold_exempt_files.keys())
        {
            if !selected.contains(name) {
                bail!("coverage configuration refers to unselected package '{name}'");
            }
        }
        if selected.is_empty() {
            bail!(
                "selected coverage scope contains no packages (package scope requires a package root)"
            );
        }
        let mut plan = Self {
            cargo_args: Vec::new(),
            roots: Vec::new(),
            exemptions: Vec::new(),
            project,
        };
        for name in &selected {
            let root = &package_roots[name];
            let default_dirs = vec!["src".to_owned()];
            for directory in config.source_dirs.get(name).unwrap_or(&default_dirs) {
                let path = root.join(directory);
                if !path.is_dir() {
                    bail!(
                        "coverage source directory does not exist: {} (package '{name}')",
                        path.display()
                    );
                }
                let path = path.canonicalize()?;
                if plan.roots.contains(&path) {
                    bail!("duplicate coverage source directory: {}", path.display());
                }
                plan.roots.push(path);
            }
            for file in config
                .threshold_exempt_files
                .get(name)
                .into_iter()
                .flatten()
            {
                let path = root.join(file);
                if !path.is_file() {
                    bail!(
                        "coverage exemption file does not exist: {} (package '{name}')",
                        path.display()
                    );
                }
                plan.exemptions.push(path.canonicalize()?);
            }
        }
        if scope == "package" {
            for name in &selected {
                plan.cargo_args
                    .extend(["--package".to_owned(), name.clone()]);
            }
        } else {
            plan.cargo_args.push("--workspace".to_owned());
            for name in package_roots
                .keys()
                .filter(|name| !selected.contains(*name))
            {
                plan.cargo_args
                    .extend(["--exclude".to_owned(), name.clone()]);
            }
        }
        Ok(plan)
    }

    /// Resolves an absolute or project-relative report filename.
    ///
    /// Existing paths are canonicalized so aliases match configured paths;
    /// reports for unavailable source files retain their absolute path.
    pub(crate) fn report_path(&self, filename: &str) -> PathBuf {
        let path = self.project.join(filename);
        fs::canonicalize(&path).unwrap_or(path)
    }
}
