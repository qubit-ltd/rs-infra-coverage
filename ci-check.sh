#!/usr/bin/env bash
set -euo pipefail
trap 'status=$?; if [ "$status" -eq 0 ]; then echo "rs-infra-coverage ci-check: succeeded"; else echo "rs-infra-coverage ci-check: failed (exit code $status)" >&2; fi; exit "$status"' EXIT
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
