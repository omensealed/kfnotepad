# Changelog

Notable user-facing and engineering changes are recorded here. kfnotepad follows semantic versioning while it is in
early development; minor releases may substantially revise internal architecture while preserving documented file
safety and privacy behavior.

## Unreleased

### Engineering

- Added a bounded per-pane GUI visual-row layout cache keyed by document revision, viewport, body width, and wrapping
  mode. Rendering and pointer hit-testing now share the same row ranges while current selection and syntax styling
  remain uncached.
- Removed the GUI's duplicate Iced full-text mirror. The shared `TextBuffer` is now the canonical editor text state,
  while the GUI adapter retains only interaction and viewport metadata.
- Removed the dormant native-Iced GUI editor branch and its hardcoded backend selector. The app-owned editor renderer
  is now the sole GUI editing path.

## 0.2.0 - 2026-07-13

### Highlights

- Reorganized the core, terminal, and GUI implementations into focused modules with thin binary launchers.
- Replaced full-document undo snapshots with byte-budgeted insert, delete, replace, and compound edit deltas.
- Added coalesced typing and paste history while keeping undo and redo bounded by entry count and retained bytes.
- Moved GUI file-tree loading and recursive expansion out of Iced view construction and into stale-safe background
  tasks backed by a cached row model.
- Added a long-lived, debounced GUI file watcher with conservative snapshot revalidation and metadata polling fallback.
- Removed full document/history clones from asynchronous GUI saves and coalesced repeated save requests per tile.
- Reduced full-buffer reconstruction during GUI overwrite, IME, Cut, Paste, selection, search, and page-navigation
  paths.
- Made syntax highlighting independently optional for TUI and GUI source builds. Default TUI builds and native
  release packages continue to include syntax highlighting.

### Correctness And Resource Use

- Enforced the 8 MiB text limit during editing and paste, before oversized content reaches save.
- Added exact delta replay and model-based tests for ASCII, Unicode, multiline, grapheme, trailing-newline, and mixed
  undo/redo histories.
- Added reproducible core text benchmarks and release binary-size measurements.
- Kept external-change protection, UTF-8 validation, symlink/non-regular rejection, atomic save behavior, and dirty
  buffer protection intact throughout the refactor.

### Build And Release

- Expanded CI and local feature checks across core-only, lean and syntax-enabled TUI/GUI, and all-feature profiles.
- Kept terminal-only dependency graphs free of Iced and native dialog dependencies.
- Pinned and checksum-verified the AppImage packaging tool and embedded runtime used by release automation.
- Continued producing Linux tarball, Debian, and AppImage artifacts, Windows executables/ZIP, and an unsigned macOS
  disk image from version tags.

## 0.1.1 - 2026-07-11

- Published the first cross-platform alpha release with separate Crossterm TUI and Iced GUI binaries.
- Established the local-only UTF-8 file contract, conservative atomic saves, workspace support, managed notes,
  operating-system Trash/Recycle Bin deletion, and Linux/Windows/macOS release packaging.
