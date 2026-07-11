#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build --locked --no-default-features --features tui --bin kfnotepad
cargo build --locked --no-default-features --features gui --bin kfnotepad-gui
cargo build --locked --all-targets --all-features
