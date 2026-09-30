#!/usr/bin/env bash
set -euo pipefail
project_root=${project_root:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)}
cleanup_build_artifacts() { local status=$? path; for path in "$project_root/target/debug" "$project_root/target/release" "$project_root/target/tmp" "$project_root/target/llvm-cov-target" "$project_root/target/infra-feature-matrix"; do if [ -d "$path" ]; then echo "Cleaning transient build artifacts: $path"; command rm -rf -- "$path" || status=1; fi; done; for path in "$project_root"/.infra/*/tool.revision.tmp; do [ ! -f "$path" ] || command rm -f -- "$path" || status=1; done; echo "Build artifact cleanup completed"; return "$status"; }
if [ -z "${RS_INFRA_CLEANUP_OWNER_PID:-}" ]; then export RS_INFRA_CLEANUP_OWNER_PID=$$; trap cleanup_build_artifacts EXIT; fi
