# Contributing to kfnotepad

Read `README.md`, `docs/README.md`, and the relevant public contract docs before changing code. Keep changes focused,
add or update tests for behavior changes, and run `./scripts/check.sh`.

For feature-gated work, `./scripts/check.sh` now also runs:

- Core-only (`--no-default-features`)
- TUI-only (`--no-default-features --features tui`)
- GUI-only (`--no-default-features --features gui`)
- All-features coverage

Run `cargo bench --locked --no-default-features --bench core_text` when changing text-buffer, search, or undo
behavior. Benchmark data must remain synthetic and should be recorded with host and compiler details.

CI now runs this check matrix across `ubuntu-latest`, `macos-latest`, and `windows-latest`.

Do not include secrets, production data, generated databases, build outputs, or unrelated formatting changes.
Document the rationale before changing architecture, persistent formats, public interfaces, or production dependencies.
