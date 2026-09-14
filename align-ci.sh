#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all
cargo fmt --all -- --check
