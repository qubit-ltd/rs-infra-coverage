use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rs-infra-coverage")]
struct Cli {
    #[arg(long, default_value = ".")]
    project: PathBuf,
    #[arg(long, default_value = ".infra/ci/coverage.json")]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Collect {
        #[arg(long)]
        output: Option<PathBuf>,
    },
    Check {
        #[arg(long)]
        input: PathBuf,
    },
    Report {
        #[arg(long)]
        input: PathBuf,
    },
    Clippy {
        #[arg(long)]
        coverage_cfg: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let project = std::fs::canonicalize(&cli.project)?;
    let configured_path = if cli.config.is_absolute() {
        cli.config
    } else {
        project.join(cli.config)
    };
    let config_path = qubit_infra_coverage::resolve_config_path(&project, &configured_path)?;
    match cli.command {
        Command::Collect { output } => {
            qubit_infra_coverage::collect(&project, &config_path, output.as_deref())
        }
        Command::Check { input } => qubit_infra_coverage::check(&project, &config_path, &input),
        Command::Report { input } => qubit_infra_coverage::report(&project, &config_path, &input),
        Command::Clippy { coverage_cfg } => {
            qubit_infra_coverage::clippy(&project, &config_path, coverage_cfg)
        }
    }
}
