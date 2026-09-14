# rs-infra-coverage

[![Rust CI](https://github.com/qubit-ltd/rs-infra-coverage/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-infra-coverage/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-infra-coverage/coverage-badge.json)](https://qubit-ltd.github.io/rs-infra-coverage/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-infra-coverage.svg?color=blue)](https://crates.io/crates/qubit-infra-coverage)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Collect LLVM coverage data, evaluate project coverage thresholds, and run the
coverage-specific Clippy compatibility check from configuration.

## Installation

```bash
cargo install --git https://github.com/qubit-ltd/rs-infra-coverage.git --tag v0.1.0 qubit-infra-coverage
```

## Quick Start

From a Rust project root:

```bash
cargo run --manifest-path /path/to/rs-infra-coverage/Cargo.toml -- --help
```

The project's `.infra` configuration remains the source of truth; this tool does not copy project configuration into the tool repository.

## Commands

```bash
rs-infra-coverage --project . collect
rs-infra-coverage --project . check --input target/infra/coverage/raw.json
rs-infra-coverage --project . report --input target/infra/coverage/raw.json
rs-infra-coverage --project . clippy --coverage-cfg
```

`collect` supports `scope` (`default-members`, `workspace`, or `package`) and
`exclude_packages`. `check` applies configured `thresholds` after removing
`threshold_exempt_files`. If `.infra/ci/coverage.json` is absent, the legacy
`.rs-ci-coverage.json` is accepted with a migration warning.

The Clippy command enables `RUSTFLAGS=--cfg coverage` when `--coverage-cfg`,
`RUN_COVERAGE_CFG_CLIPPY=1`, `coverage_cfg_clippy: true`, or
`clippy.coverage_cfg: true` is configured.

## Capabilities and limitations

Project-specific policy belongs in `.infra`, and orchestration belongs in
`rs-infra-ci`. Compatibility is limited to the commands and configuration
fields documented above.

## Learn More

See the command help and source tests for the supported interface. Switch to [中文文档](README.zh_CN.md).

## Testing

```bash
cargo test
cargo test --all-features
./ci-check.sh
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public API documentation and tests current, and run `./align-ci.sh` to format code and `./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-infra-coverage](https://github.com/qubit-ltd/rs-infra-coverage)
