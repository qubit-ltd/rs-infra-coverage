#!/usr/bin/env bash
set -euo pipefail
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
