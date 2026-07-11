# Contributing to kfnotepad

Read `README.md`, `docs/README.md`, and the relevant public contract docs before changing code. Keep changes focused,
add or update tests for behavior changes, and run `./scripts/check.sh`.

For feature-gated work, `./scripts/check.sh` now also runs:

- TUI-only (`--no-default-features --features tui`)
- GUI-only (`--no-default-features --features gui`)
- All-features coverage

CI now runs this check matrix across `ubuntu-latest`, `macos-latest`, and `windows-latest`.

Do not include secrets, production data, generated databases, build outputs, or unrelated formatting changes.
Document the rationale before changing architecture, persistent formats, public interfaces, or production dependencies.
