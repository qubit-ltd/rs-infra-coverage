#!/usr/bin/env bash
set -euo pipefail
trap 'status=$?; if [ "$status" -eq 0 ]; then echo "rs-infra-coverage coverage: succeeded"; else echo "rs-infra-coverage coverage: failed (exit code $status)" >&2; fi; exit "$status"' EXIT
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
