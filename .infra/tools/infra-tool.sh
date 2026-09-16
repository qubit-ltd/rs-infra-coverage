#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
project_root=$(cd "$script_dir/../.." && pwd -P)
config="$project_root/.infra/ci/tools.toml"
install_root="$project_root/.infra/tools/bin"
bin_dir="$install_root/bin"
die() { echo "error: $*" >&2; exit 1; }
[ $# -ge 1 ] || die "usage: infra-tool.sh TOOL [ARGS...]"
tool="$1"; shift
value() { awk -v section="[$tool]" -v key="$1" '$0 == section { in_section=1; next } /^\[/ { in_section=0 } in_section && $0 ~ "^[[:space:]]*" key "[[:space:]]*=" { value=$0; sub(/^[^=]*=[[:space:]]*"/, "", value); sub(/"[[:space:]]*$/, "", value); print value; exit }' "$config"; }
source=$(value source); revision=$(value revision); binary=$(value binary); package=$(value package)
[ -n "$source" ] && [ -n "$revision" ] && [ -n "$binary" ] && [ -n "$package" ] || die "incomplete tool configuration for $tool"
mkdir -p "$bin_dir"
target="$bin_dir/$binary"; marker="$install_root/$tool.revision"; installed=""
[ -f "$marker" ] && IFS= read -r installed < "$marker" || true
if [ ! -x "$target" ] || [ "$installed" != "$revision" ]; then cargo install --git "$source" --rev "$revision" --locked --force --root "$install_root" "$package" --bin "$binary"; printf '%s\n' "$revision" > "$marker.tmp"; mv "$marker.tmp" "$marker"; fi
if [ "$tool" = "rs-infra-ci" ]; then for dependency in rs-infra-style rs-infra-verify rs-infra-coverage; do "$script_dir/infra-tool.sh" "$dependency" --help >/dev/null; done; fi
exec env RS_INFRA_BIN_DIR="$bin_dir" "$target" "$@"
