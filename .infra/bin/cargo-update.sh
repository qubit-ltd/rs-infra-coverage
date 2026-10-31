#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)

command -v cargo >/dev/null 2>&1 || {
    echo "error: Cargo is required to update project dependencies" >&2
    exit 2
}

echo "Updating Cargo dependencies in $project_root"
(cd "$project_root" && cargo update)

while IFS= read -r -d '' manifest; do
    [[ "$manifest" == "$project_root/Cargo.toml" ]] && continue
    crate_dir=${manifest%/Cargo.toml}
    echo "Updating Cargo dependencies in $crate_dir"
    cargo update --manifest-path "$manifest"
done < <(
    find "$project_root" \
        -type d \( \
            -name .git -o \
            -name .infra -o \
            -name target -o \
            -name build -o \
            -name dist -o \
            -name out -o \
            -name node_modules -o \
            -name artifacts -o \
            -name coverage -o \
            -name vendor \
        \) -prune -o \
        -type f -name Cargo.toml -print0
)
