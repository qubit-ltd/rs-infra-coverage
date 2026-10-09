#!/usr/bin/env bash
set -euo pipefail

project_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
mode=${1:-}
if [[ "$mode" != "" && "$mode" != --yes && "$mode" != --dry-run && "$mode" != --status && "$mode" != --check ]]; then
    echo "usage: ./update-infra.sh [--yes|--dry-run|--status|--check]" >&2
    exit 2
fi
python_cmd=${RS_INFRA_PYTHON:-python3}
tmp_root=${TMPDIR:-/tmp}
work=$(mktemp -d "$tmp_root/rs-infra-bootstrap.XXXXXX")
printf '%s\n' 'rs-infra-bootstrap-temp-v1' > "$work/.rs-infra-bootstrap-temp"
cleanup() { rm -rf -- "$work"; }
trap cleanup EXIT
repository=https://github.com/qubit-ltd/rs-infra-tools.git
if ! git clone --quiet --depth 1 --single-branch --branch main "$repository" "$work/source"; then
    echo "error: unable to fetch the latest rs-infra-tools bootstrap package" >&2
    exit 1
fi
export RS_INFRA_SOURCE_REVISION
RS_INFRA_SOURCE_REVISION=$(git -C "$work/source" rev-parse HEAD)
if command -v cygpath >/dev/null 2>&1; then
    export RS_INFRA_BOOTSTRAP_TEMP_ROOT=$(cygpath -aw "$work")
else
    export RS_INFRA_BOOTSTRAP_TEMP_ROOT="$work"
fi
exec "$python_cmd" "$work/source/assets/project-bootstrap/sync.py" \
    --project-root "$project_root" \
    --package-root "$work/source/assets/project-bootstrap" \
    --configs-only "$@"
