#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
profile_args=(--release)
if [[ "${KFNOTEPAD_DEBUG_RUN:-}" == "1" ]]; then
  profile_args=()
fi
cargo run --locked "${profile_args[@]}" -- "$@"
