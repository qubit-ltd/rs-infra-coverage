#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
project_root=$(cd "$script_dir/../.." && pwd -P)
source "$project_root/.infra/lib/cleanup-build-artifacts.sh"
tool="${1:-}"; [ -n "$tool" ] || { echo 'usage: infra-tool.sh rs-infra-TOOL [ARGS...]' >&2; exit 2; }; shift
case "$tool" in rs-infra-ci|rs-infra-coverage|rs-infra-dependency|rs-infra-pages|rs-infra-style|rs-infra-verify|rs-infra-tools) ;; *) echo "error: unknown infra tool '$tool'" >&2; exit 2 ;; esac
name="${tool#rs-infra-}"; tool_root="$project_root/.infra/$name"; config="$tool_root/tool.toml"; bin_dir="$tool_root/bin"; target="$bin_dir/$tool"; marker="$tool_root/tool.revision"
[ -f "$config" ] || { echo "error: tool configuration not found: $config" >&2; exit 1; }
value() { awk -F '"' -v key="$1" '$0 ~ "^[[:space:]]*" key "[[:space:]]*=" { print $2; exit }' "$config"; }
source=$(value source); revision=$(value revision); binary=$(value binary); package=$(value package)
[ -n "$source" ] && [ -n "$revision" ] && [ -n "$binary" ] && [ -n "$package" ] || { echo "error: incomplete tool configuration: $config" >&2; exit 1; }
mkdir -p "$bin_dir"
if [ ! -x "$bin_dir/$binary" ] || [ "$(cat "$marker" 2>/dev/null || true)" != "$revision" ]; then PATH="$bin_dir:${PATH:-}" cargo install --git "$source" --rev "$revision" --locked --force --root "$tool_root" "$package" --bin "$binary"; printf '%s\n' "$revision" > "$marker.tmp"; mv "$marker.tmp" "$marker"; fi
all_bins="$project_root/.infra/ci/bin:$project_root/.infra/coverage/bin:$project_root/.infra/dependency/bin:$project_root/.infra/pages/bin:$project_root/.infra/style/bin:$project_root/.infra/verify/bin:$project_root/.infra/tools/bin"
env -u RS_INFRA_BIN_DIR PATH="$all_bins:${PATH:-}" "$bin_dir/$binary" "$@"
