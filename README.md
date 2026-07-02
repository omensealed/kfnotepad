# kfnotepad

Local UTF-8 text-file editor with a modern terminal UI and a separate Iced GUI review path.

## Status

kfnotepad is a local-only UTF-8 text editor with a terminal UI and a separate Iced GUI. The current public
contracts, security notes, and operations runbook are in `docs/`.

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
cargo run --locked --release --bin kfnotepad-gui -- path/to/note.txt
```

The GUI opens local files as tiled documents and uses the same file validation/save adapter as the terminal editor.
See [`docs/17-GUI-CONTRACT.md`](docs/17-GUI-CONTRACT.md) for GUI controls, persistence, current gaps, and smoke steps.

Local release artifact:

```bash
./scripts/package.sh
```

This writes a tarball, Linux `.deb`, AppImage, and SHA-256 files under ignored `dist/` with both `kfnotepad` and
`kfnotepad-gui`. Packaging and verification notes are in [`docs/13-OPERATIONS.md`](docs/13-OPERATIONS.md).

## Selected direction

- Type: cli plus separate GUI binary
- Stage: local TUI baseline complete; Iced GUI review path documented
- Stack: rust, shell, iced
- Database: none; normal files on disk
- Platforms: cachyos-linux
- License: AGPL-3.0-or-later

## Documentation

See [`docs/README.md`](docs/README.md) for the public documentation map.
