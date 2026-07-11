#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

printf '%s\n' '== Feature matrix checks =='

printf '%s\n' '- TUI-only build (no GUI deps):'
cargo check --locked --no-default-features --features tui
cargo test --locked --no-default-features --features tui

printf '%s\n' '- GUI-only build (no TUI build):'
cargo check --locked --no-default-features --features gui
cargo test --locked --no-default-features --features gui

printf '%s\n' '- All features build/tests:'
cargo check --locked --all-features --all-targets
cargo test --locked --all-features
