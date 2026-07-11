# Performance baselines

Performance work must use release builds and record the host, Rust version, enabled features, input shape, and exact
command. Keep document contents synthetic and paths disposable so benchmark output cannot expose user data.

## Reference commands

```bash
rustc -Vv
cargo tree -e features --no-default-features --features tui
cargo tree -e features --no-default-features --features "tui syntax"
cargo tree -e features --no-default-features --features gui
cargo tree -e features --all-features
/usr/bin/time -v cargo build --locked --release --all-features --bins
```

GUI idle-I/O measurements should use four synthetic 8 MiB files for five minutes with reader mode disabled. Record
full-file snapshot reads, metadata checks, process CPU time, and peak RSS. Save measurements should separately record
text serialization, conflict validation, atomic write, and completion handling. Do not infer improvement from elapsed
time alone; count the expensive operations under test.

## Initial environment

- Date: 2026-07-11
- Host: x86_64-unknown-linux-gnu
- Rust: 1.97.0 (2d8144b78 2026-07-07)
- Cargo: 1.97.0 (c980f4866 2026-06-30)
- File limit: 8 MiB

The initial static baseline confirms that each GUI external-change tick schedules one snapshot operation per open
tile, and each snapshot reads and fingerprints the complete file. Timed and allocation baselines will be added beside
the focused benchmark harness before changing the snapshot/save implementations.

## External-change polling improvement

The first polling correction kept the one-second responsiveness contract but compared symlink-safe file metadata
before requesting a full snapshot. It also added an in-flight guard to prevent overlapping scans.

The next correction adds one long-lived `notify-debouncer-mini` service. It watches the parent directories of open
documents non-recursively and drains events without blocking the GUI. Only matching open paths receive immediate
strong snapshot validation. While the watcher is healthy, a 60-second metadata-only fallback check guards lifecycle
mistakes without rereading unchanged files. Watcher failure restores one-second metadata polling and periodic deep
verification. Save-time conflict validation remains independent and authoritative.

## File-tree rendering improvement

The GUI now owns one cached custom file-tree row model. Iced view construction renders only that cache and performs no
directory reads. Root changes, expansion, explicit refresh, create, and delete rebuild the cache; selection only
updates cached flags. The unused parallel `iced-swdir-tree` state and dependency were removed. Directory loading is
Recursive cache rebuilding now runs in a blocking worker and carries a monotonic generation; stale results cannot
replace a newer root or expansion request. The last valid cache remains renderable while a replacement loads. The
core sidebar's single-directory `load`/`refresh` operation is still synchronous and is the next browser I/O boundary.
