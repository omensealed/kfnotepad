<p align="center">
  <img src="assets/kfnotepad-logo.png" alt="kfnotepad logo" width="260">
</p>

# kfnotepad

Local UTF-8 text-file editor with a modern terminal UI and a separate Iced GUI review path.

## Status

kfnotepad is a local-only UTF-8 text editor with a terminal UI and a separate Iced GUI. The current public
contracts, security notes, and operations runbook are in `docs/`.

Safety defaults include regular UTF-8 file handling, symlink/non-regular file rejection, atomic saves, and
file-browser deletion through the operating system Trash/Recycle Bin after explicit confirmation.

## Start here

```bash
./scripts/doctor.sh
./scripts/check.sh
./scripts/run.sh --help
```

Current editor launch:

```bash
./scripts/run.sh path/to/note.txt
```

In an interactive terminal this opens the editor. In non-interactive contexts it prints a read-only summary.
See [`docs/16-CLI-CONTRACT.md`](docs/16-CLI-CONTRACT.md) for keybindings and current behavior.

Current GUI launch:

```bash
cargo run --locked --no-default-features --features gui --release --bin kfnotepad-gui -- path/to/note.txt
```

The GUI opens local files as tiled documents and uses the same file validation/save adapter as the terminal editor.
See [`docs/17-GUI-CONTRACT.md`](docs/17-GUI-CONTRACT.md) for GUI controls, persistence, current gaps, and smoke steps.

Local release artifact:

```bash
./scripts/package.sh
```

This writes a tarball, Linux `.deb`, AppImage, and SHA-256 files under ignored `dist/` with both `kfnotepad` and
`kfnotepad-gui`. Packaging and verification notes are in [`docs/13-OPERATIONS.md`](docs/13-OPERATIONS.md).

Version tags matching `vX.Y.Z` run the native GitHub release workflow. Releases include Linux packages, standalone
Windows TUI/GUI `.exe` files plus a combined ZIP, and an unsigned macOS `.dmg` containing `kfnotepad.app` and the
terminal binary. Published artifacts are covered by a consolidated `SHA256SUMS` file.

## Selected direction

- Type: cli plus separate GUI binary
- Stage: local TUI baseline complete; Iced GUI review path documented
- Stack: rust, shell, iced
- Database: none; normal files on disk
- Support tiers:
  - Supported: Linux (primary; CI + local packaging documented).
  - Windows/macOS: native builds and tests run in CI; unsigned alpha packages are produced by the release workflow.
- License: AGPL-3.0-or-later

### Build features

The crate supports feature gates for leaner builds:

- `tui` (default): terminal editor dependencies
- `gui`: Iced GUI dependencies for the separate GUI binary
- `syntax`: syntax highlighting dependency

Default features are `["tui", "syntax"]`. GUI is intentionally excluded from defaults so terminal-only users can build without Iced/rfd.

Default builds are terminal-capable. For an explicit minimal TUI-only build in constrained environments, run:

```bash
cargo build --locked --no-default-features --features tui
cargo run --locked --no-default-features --features tui --bin kfnotepad -- --help
```

To build the GUI binary explicitly:

```bash
cargo build --locked --no-default-features --features gui
cargo run --locked --no-default-features --features gui --release --bin kfnotepad-gui -- --describe
```

External file changes are detected by conservative polling and save-time conflict checks. A filesystem watcher feature is intentionally not exposed until it can be implemented as a long-lived, nonblocking watcher.

## Documentation

See [`docs/README.md`](docs/README.md) for the public documentation map.

## Storage and paths

Editor config, notes, and layout/workspace persistence use platform config/data directories resolved through the `dirs` crate:

- Config and UI layout/preferences: platform config directory + `kfnotepad/`.
- Managed notes: platform data directory + `kfnotepad/notes`.
- Workspace snapshots: platform config directory + `kfnotepad/workspaces/...`.

The behavior is intentionally deterministic and preserves prior `XDG` layout on Unix when present while allowing
non-Unix platforms to follow their native directories.
