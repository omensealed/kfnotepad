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
