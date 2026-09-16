#!/usr/bin/env bash
set -euo pipefail
trap 'status=$?; if [ "$status" -eq 0 ]; then echo "rs-infra-coverage align-ci: succeeded"; else echo "rs-infra-coverage align-ci: failed (exit code $status)" >&2; fi; exit "$status"' EXIT
cargo fmt --all
cargo fmt --all -- --check
