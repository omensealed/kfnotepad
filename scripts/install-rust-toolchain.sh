#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

toolchain="$(sed -n 's/^channel = "\([^"]*\)"/\1/p' rust-toolchain.toml)"
profile="$(sed -n 's/^profile = "\([^"]*\)"/\1/p' rust-toolchain.toml)"
components="$(sed -n 's/^components = \[\(.*\)\]/\1/p' rust-toolchain.toml | tr -d '" ' )"

if [[ -z "$toolchain" || -z "$profile" ]]; then
    printf '%s\n' 'rust-toolchain.toml must declare a channel and profile.' >&2
    exit 1
fi

args=(toolchain install "$toolchain" --profile "$profile")
if [[ -n "$components" ]]; then
    args+=(--component "$components")
fi

rustup "${args[@]}"
