#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
shellcheck scripts/*.sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
