# Contributing to kfnotepad

Read `README.md`, `docs/README.md`, and the relevant public contract docs before changing code. Keep changes focused,
add or update tests for behavior changes, and run `./scripts/check.sh`.

For feature-gated work, `./scripts/check.sh` now also runs:

- Core-only (`--no-default-features`)
- lean TUI (`--no-default-features --features tui`)
- TUI with highlighting (`--no-default-features --features "tui syntax"`)
- lean GUI (`--no-default-features --features gui`)
- GUI with highlighting (`--no-default-features --features "gui syntax"`)
- All-features coverage

Run `cargo bench --locked --no-default-features --bench core_text` when changing text-buffer, search, or undo
behavior. Benchmark data must remain synthetic and should be recorded with host and compiler details.

CI runs this check matrix across `ubuntu-24.04`, `macos-15`, and `windows-2025`.

Do not include secrets, production data, generated databases, build outputs, or unrelated formatting changes.
Document the rationale before changing architecture, persistent formats, public interfaces, or production dependencies.
