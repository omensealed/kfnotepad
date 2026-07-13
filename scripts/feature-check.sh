#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

printf '%s\n' '== Feature matrix checks =='

printf '%s\n' '- Core-only build (no front end or syntax dependencies):'
cargo check --locked --no-default-features --lib
cargo test --locked --no-default-features --lib
cargo check --locked --no-default-features --bench core_text

printf '%s\n' '- Lean TUI build (no GUI or syntax deps):'
cargo check --locked --no-default-features --features tui
cargo test --locked --no-default-features --features tui

printf '%s\n' '- TUI with syntax highlighting:'
cargo check --locked --no-default-features --features 'tui syntax'
cargo test --locked --no-default-features --features 'tui syntax'

printf '%s\n' '- Lean GUI build (no TUI or syntax deps):'
cargo check --locked --no-default-features --features gui
cargo test --locked --no-default-features --features gui

printf '%s\n' '- GUI with syntax highlighting:'
cargo check --locked --no-default-features --features 'gui syntax'
cargo test --locked --no-default-features --features 'gui syntax'

printf '%s\n' '- All features build/tests:'
cargo check --locked --all-features --all-targets
cargo test --locked --all-features
