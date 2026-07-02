#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [[ -z "${XDG_CONFIG_HOME:-}" ]]; then
  export XDG_CONFIG_HOME
  XDG_CONFIG_HOME="$(mktemp -d "${TMPDIR:-/tmp}/kfnotepad-test-config.XXXXXX")"
fi

if [[ -z "${XDG_DATA_HOME:-}" ]]; then
  export XDG_DATA_HOME
  XDG_DATA_HOME="$(mktemp -d "${TMPDIR:-/tmp}/kfnotepad-test-data.XXXXXX")"
fi

cargo test --locked
