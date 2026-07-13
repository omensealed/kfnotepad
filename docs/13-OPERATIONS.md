# Operations and maintenance

## Support tiers

- Current target and support:
  - Tier 1: Linux desktop (validated in local workflows and package tooling).
  - Tier 2: macOS and Windows (native feature-gated builds and tests run in CI; unsigned alpha packages are produced
    for tagged releases).
- CLI/GUI build paths:
  - TUI path: `--no-default-features --features tui` builds a lean plain-text terminal editor; add `syntax` for
    Syntect highlighting.
  - GUI path: `--no-default-features --features gui` builds a lean plain-text GUI; add `syntax` for highlighting.
  - Native release packages build both front ends with `syntax` enabled.
  - GUI external-change checks use a long-lived debounced native watcher, metadata-first fallback polling, and
    independent save-time conflict detection.
- CI runs checks for all supported platforms and the feature matrix described in `scripts/feature-check.sh`.

## Development runbook

```bash
./scripts/doctor.sh
./scripts/check.sh
./scripts/run.sh --help
./scripts/run.sh README.md
cargo run --locked --no-default-features --features "gui syntax" --release --bin kfnotepad-gui -- --describe
```

`./scripts/run.sh` forwards arguments to `cargo run --locked --release --` by default. In an interactive terminal,
`./scripts/run.sh FILE` opens the terminal editor loop using optimized code. Set `KFNOTEPAD_DEBUG_RUN=1` to use the
debug profile while developing. In non-interactive contexts it prints the read-only file summary instead, so scripts
and CI do not hang.

Current editor controls:

- Arrow keys move.
- Printable characters insert.
- Enter splits a line.
- Backspace deletes the previous user-perceived character/grapheme cluster before the cursor.
- Delete removes the user-perceived character/grapheme cluster at the cursor or joins the next line at line end.
- File-browser Delete moves confirmed files/directories to the operating system Trash/Recycle Bin where supported.
- Ctrl-Z undoes edits since the last save.
- Ctrl-F searches text; Enter accepts the query and Esc cancels.
- Ctrl-L toggles line numbers for the current editor session.
- Ctrl-T cycles built-in themes for the current editor session.
- Ctrl-S saves through the tested save adapter.
- Ctrl-Q quits; dirty buffers require Ctrl-Q twice.

Known code file extensions are syntax-highlighted in the interactive editor through the bundled `syntect` syntax
set. Unknown files and plain text fall back to normal body rendering.

Theme, line-number, and wrap choices persist under the platform config directory at `.../kfnotepad/config.toml`. On Unix
that maps to `$XDG_CONFIG_HOME/kfnotepad/config.toml` or `$HOME/.config/kfnotepad/config.toml` as a fallback. The separate GUI
binary shares the same preference file.

The public CLI/keybinding contract and manual terminal checklist live in `docs/16-CLI-CONTRACT.md`. The separate Iced
GUI contract, controls, manual smoke, and current feature gaps live in `docs/17-GUI-CONTRACT.md`.

## GUI runbook

The GUI is a separate local review binary, not a replacement for the terminal editor:

```bash
cargo run --locked --no-default-features --features "gui syntax" --release --bin kfnotepad-gui -- --help
cargo run --locked --no-default-features --features "gui syntax" --release --bin kfnotepad-gui -- --describe
cargo run --locked --no-default-features --features "gui syntax" --release --bin kfnotepad-gui -- README.md
```

Use a disposable config directory for manual verification when possible:

```bash
tmpdir=$(mktemp -d)
printf 'first\n' > "$tmpdir/first.txt"
printf 'second\n' > "$tmpdir/second.txt"
XDG_CONFIG_HOME="$tmpdir/config" cargo run --locked --no-default-features --features "gui syntax" --release --bin kfnotepad-gui -- "$tmpdir/first.txt" "$tmpdir/second.txt"
rm -rf "$tmpdir"
```

The GUI opens each valid file as a tile. It shares the TUI open/save validation and atomic save behavior, has no
database or network behavior, and stores only display preferences plus geometry-only GUI layout under the platform config
directory.

For local X11 visual smoke, run:

```bash
./scripts/gui-visual-smoke.sh
```

This uses disposable files/config and writes an ignored screenshot under `target/gui-visual-smoke/`.

## Local artifact packaging

Create a local release artifact with:

```bash
./scripts/package.sh
```

The script builds release binaries, stages `bin/kfnotepad`, `bin/kfnotepad-gui`, `README.md`,
`docs/13-OPERATIONS.md`, `docs/16-CLI-CONTRACT.md`, and `docs/17-GUI-CONTRACT.md`, then writes the tarball plus
Linux package artifacts:

```text
dist/kfnotepad-0.1.1-cachyos-linux-x86_64.tar.gz
dist/kfnotepad-0.1.1-cachyos-linux-x86_64.tar.gz.sha256
dist/kfnotepad_0.1.1_amd64.deb
dist/kfnotepad_0.1.1_amd64.deb.sha256
dist/kfnotepad-0.1.1-x86_64.AppImage
dist/kfnotepad-0.1.1-x86_64.AppImage.sha256
dist/SHA256SUMS
```

The `.deb` requires `dpkg-deb`. The AppImage requires `appimagetool`. For fully offline AppImage builds, set
`KFNOTEPAD_APPIMAGE_RUNTIME=/path/to/runtime-x86_64`; otherwise `appimagetool` may try to download the runtime.

The GitHub release workflow does not trust the mutable upstream `continuous` download URLs directly. It uses
`scripts/install-pinned-appimage-tools.sh` to download the x86-64 tool and runtime by GitHub release asset ID and
checks their pinned SHA-256 digests before either file is executed or embedded. Updating either pin requires reviewing
the upstream build, replacing its asset ID and digest together, and running the Linux packaging smoke from a clean
tool directory.

The script honors `CARGO_TARGET_DIR` for builds and staging. Set `KFNOTEPAD_DIST_DIR` to write artifacts outside the
default `dist/` directory, which is useful for isolated release checks.
Debian 11+ support is a build-environment commitment: build the release on Debian 11 or an equivalent older-glibc
container/runner before claiming Debian 11 compatibility. A package generated on CachyOS can be structurally valid
and work on newer Debian-derived systems, but may require newer runtime libraries than Debian 11 provides.

Verify the artifact before install testing:

```bash
sha256sum -c dist/SHA256SUMS
tar -tzf dist/kfnotepad-0.1.1-cachyos-linux-x86_64.tar.gz
dpkg-deb --info dist/kfnotepad_0.1.1_amd64.deb
dpkg-deb --contents dist/kfnotepad_0.1.1_amd64.deb
```

`dist/` is ignored because these files are generated local artifacts.

The AppImage launches the GUI by default. For the terminal editor, pass `--cli`:

```bash
dist/kfnotepad-0.1.1-x86_64.AppImage --help
dist/kfnotepad-0.1.1-x86_64.AppImage --cli --help
```

If FUSE is unavailable in the test environment, extract and run `AppRun` directly:

```bash
tmpdir=$(mktemp -d)
cd "$tmpdir"
/path/to/kfnotepad-0.1.1-x86_64.AppImage --appimage-extract
squashfs-root/AppRun --help
squashfs-root/AppRun --cli --help
```

## Local install lifecycle

The tarball is portable as a user-owned prefix install. Do not use `sudo` for the local verification path.

Fresh install:

```bash
prefix="$HOME/.local/kfnotepad-0.1.1"
mkdir -p "$prefix"
tar -xzf dist/kfnotepad-0.1.1-cachyos-linux-x86_64.tar.gz -C "$prefix" --strip-components=1
"$prefix/bin/kfnotepad" --version
"$prefix/bin/kfnotepad-gui" --version
"$prefix/bin/kfnotepad-gui" --describe
```

Upgrade or reinstall the same version by extracting a verified tarball to a new temp directory, preserving the
previous binary as a rollback copy, then replacing the old prefix contents:

```bash
cp "$prefix/bin/kfnotepad" "$prefix/bin/kfnotepad.previous"
cp "$prefix/bin/kfnotepad-gui" "$prefix/bin/kfnotepad-gui.previous"
tar -xzf dist/kfnotepad-0.1.1-cachyos-linux-x86_64.tar.gz -C "$prefix" --strip-components=1
"$prefix/bin/kfnotepad" --version
"$prefix/bin/kfnotepad-gui" --version
```

Rollback is binary replacement because kfnotepad has no schema, managed data directory, or migration state:

```bash
mv "$prefix/bin/kfnotepad.previous" "$prefix/bin/kfnotepad"
mv "$prefix/bin/kfnotepad-gui.previous" "$prefix/bin/kfnotepad-gui"
"$prefix/bin/kfnotepad" --version
"$prefix/bin/kfnotepad-gui" --version
```

Uninstall the user-owned prefix:

```bash
rm -rf "$prefix"
```

Verified on 2026-06-26 in a disposable temp prefix: the local package contained both `bin/kfnotepad` and
`bin/kfnotepad-gui`; checksum verification passed; extracted `kfnotepad --version`, `kfnotepad-gui --version`, and
`kfnotepad-gui --describe` passed; and an extracted `kfnotepad-gui` disposable two-file launch stayed open until the
bounded 5-second timeout, which is expected for the windowed smoke. Earlier TUI-only fresh install, same-version
upgrade, uninstall cleanup, and rollback were verified on 2026-06-24.

## Current alpha upload manifest

Version tags matching the Cargo version (`v0.1.1`, for example) trigger `.github/workflows/release.yml`. The workflow
builds on native GitHub-hosted runners and publishes:

- Linux x86-64 tarball, `.deb`, and AppImage.
- Standalone Windows x86-64 `kfnotepad.exe` and `kfnotepad-gui.exe`, plus a ZIP containing both and documentation.
- A macOS arm64 `.dmg` containing `kfnotepad.app`, the standalone terminal binary, and documentation.
- One `SHA256SUMS` covering every uploaded package.

The release workflow can also be dispatched manually for an existing `vX.Y.Z` tag. It refuses a tag whose version
does not match `Cargo.toml`. GitHub-hosted runners already contain the native Windows and macOS packaging tools; no
developer workstation tools are required for normal releases. The macOS bundle is ad-hoc signed but not Apple
notarized, so Gatekeeper may require an explicit user approval. Public notarized distribution requires an Apple
Developer ID certificate and App Store Connect credentials stored as GitHub Actions secrets.

Upload only the artifacts listed in `dist/SHA256SUMS` from the same local packaging run. The human reported a Linux Mint
package smoke passing on 2026-07-02. Do not upload older local artifacts unless `dist/SHA256SUMS` is intentionally
regenerated for them. Debian 11 package compatibility remains unclaimed until a working Bullseye build container or
runner verifies it.

## Data operations

Database mode: **none**

Notes remain normal files on disk. There is no schema, migration, backup, restore, or local database file.

Managed notes remain normal files under the platform data directory at `.../kfnotepad/notes`. On Unix this is
`$XDG_DATA_HOME/kfnotepad/notes` or `$HOME/.local/share/kfnotepad/notes` when `XDG_DATA_HOME` is unset. Use disposable
`XDG_DATA_HOME` values for manual verification; do not manually create or delete production notes as part of release
verification.

The write-safety and recovery policy is: same-directory temp file, flush, atomic rename, symlink path rejection,
non-regular file rejection, existing permission preservation, `0o600` for new Unix files, 8 MiB file limit, no
automatic backup files, and best-effort temp cleanup on failure. This path is exposed through save commands in both
front ends and covered by adapter tests using disposable development data.

Recovery expectations:

- If open validation fails, the target file is not modified.
- If save validation or temp-file creation fails, the target file is expected to remain untouched.
- If a temporary `.kfnotepad-*.tmp` sibling remains after an interrupted process, inspect it manually before deleting
  it; the app does not treat temp files as authoritative recovery data.
- After a successful Ctrl-S, kfnotepad does not keep an in-app previous version. Restore overwritten content from
  normal filesystem snapshots, backups, or version control.

## Logging and diagnostics

- Log operation identifiers and useful context, not secrets or full sensitive payloads.
- Separate user-facing messages from diagnostic detail.
- Bound log retention and file growth for long-running applications.
- Add a documented diagnostic command or checklist that does not expose credentials.

## Dependency maintenance

Review updates in small batches. Read release notes, inspect lockfile deltas, run the complete gate, and avoid
combining major upgrades with feature work. Record compatibility changes and rollback steps.

## GUI layout recovery

GUI layout persistence is under `.../kfnotepad/gui-layout.v1` in the platform config directory. On Unix this maps to
`$XDG_CONFIG_HOME/kfnotepad/gui-layout.v1` with `$HOME/.config/kfnotepad/gui-layout.v1` as a fallback. The format is
geometry-only and must not contain document text, absolute file paths, cursor positions, search queries, recent-file
history, or unsaved buffers. It may contain browser visibility, browser width, pane split ratios, pane ordinals, and
minimized ordinals.
If the GUI opens with an unwanted tile arrangement or sidebar width, close kfnotepad and delete `gui-layout.v1`; the
GUI falls back to its launch-time layout defaults.

To reset all kfnotepad display state while leaving notes and edited files alone:

```bash
rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/kfnotepad/config.toml"
rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/kfnotepad/gui-layout.v1"
```

## Incident checklist

1. Stop further damage without destroying evidence.
2. Preserve relevant redacted logs and exact versions.
3. Revoke/rotate exposed credentials outside the repository.
4. Restore from a verified backup or roll back through the documented path.
5. Add a regression test and update the relevant public docs.
6. Update threat model, operations docs, and release criteria.
