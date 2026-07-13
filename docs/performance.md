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
cargo bench --locked --no-default-features --bench core_text
```

The `core_text` Divan harness uses synthetic content and measures near-limit construction, serialization, end/missing
searches, 100 KiB paste, delete, and overwrite-paste operations, 1,000 typed ASCII inserts, and 200 undo
operations. Filter a benchmark by appending its name, for example
`cargo bench --locked --no-default-features --bench core_text -- paste_100_kib`. Record complete output with the host
and compiler details above; benchmark numbers are not portable between machines.

Divan `0.1.21` is pinned as a development-only dependency under its MIT/Apache-2.0 dual license. It is not linked into
either application binary. Its nine locked transitive packages provide its CLI, macros, terminal sizing, and matching;
none enter normal dependency graphs. Its removal path is deleting the `core_text` benchmark target and dev dependency.

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

## Core text baseline

Captured 2026-07-13 on Linux 7.1.3-2-cachyos, an Intel Xeon E5-2690 v4, and Rust 1.97.0. The short comparison run used
`--sample-count 10 --max-time 0.5`; use the default longer run for optimization decisions.

| Operation | Median |
| --- | ---: |
| Construct near-8 MiB text | 16.40 ms |
| Serialize near-8 MiB text | 1.63 ms |
| Search for marker at end | 20.58 ms |
| Search for missing text | 21.08 ms |
| Paste 100 KiB into 1 MiB text | 314.60 ms |
| Type 1,000 ASCII characters | 525.70 us |
| Undo 200 prepared operations | 10.24 us |

The paste baseline includes the current snapshot-based undo cost and is the first optimization target. These values
are local comparison data, not performance guarantees.

## Bulk insertion improvement

Captured 2026-07-13 on the same host and compiler with
`cargo bench --locked --no-default-features --bench core_text -- paste_100_kib --sample-count 10 --max-time 0.5`.
The 100 KiB paste median fell from 314.60 ms to 123.3 us.

Standalone bulk insertion now records a byte-budgeted delta containing the inserted text and exact range instead of
cloning the document into undo history. Undo removes the exact inserted range without expanding across adjacent
grapheme clusters, and redo reapplies the stored text. For the benchmark's 1 MiB document and 100 KiB paste, history
payload therefore scales with roughly 100 KiB of inserted text instead of the pre-edit document.

The remaining elapsed-time improvement comes from avoiding a full-line Unicode grapheme scan when the inserted final
segment and following byte are both ASCII (or the insertion ends the line). Unicode text and non-ASCII adjacency keep
the full grapheme-boundary normalization path. Focused tests cover multiline text, combining marks, trailing newlines,
redo invalidation, compound-edit fallback, and history-byte accounting. The result is a local comparison, not a
portable performance guarantee.

Contiguous typed characters now coalesce into the same insert-delta representation. The short 1,000-character
benchmark moved from 525.70 us to 492.8 us median; its empty starting document did not carry a meaningful snapshot
cost, so this is not treated as a material throughput result. The resource benefit appears on large documents: each
typing group now retains its inserted text and allocation capacity instead of cloning the document, allowing the
entry-count limit to remain useful without exhausting the byte budget first.

Character, word, selection, and line-boundary deletion now share an exact delete delta. A delta retains removed text,
its original range, and trailing-newline policy; undo reinserts it and redo removes the same raw range. An ASCII-line
fast path skips Unicode segmentation because every character boundary in the line is already a grapheme boundary. On
the same host, the 1 MiB structural debug test fell from about
65 seconds to 0.35 seconds of test execution. A release comparison captured with
`cargo bench --locked --no-default-features --bench core_text -- delete_100_kib --sample-count 10 --max-time 0.5`
measured a 1.068 ms median for deleting 100 KiB from a 1 MiB ASCII line. No pre-change release baseline exists for
that added benchmark, so the result is a forward baseline rather than an improvement ratio.

Standalone newline insertion now uses the same exact insert delta as bulk text, and character overwrite uses a
replacement delta containing only the removed grapheme, inserted character, and their exact ranges. Undo and redo
therefore preserve multi-code-point graphemes and trailing-newline state without retaining the surrounding document.
Compound edits store ordered delta groups and remain one user-visible undo step. Contiguous operations coalesce into
one replacement delta, while noncontiguous operations undo in reverse and redo forward. Group payload and vector
storage count toward the same 64 MiB history ceiling during assembly. A structural test verifies that a compound
replacement in a 1 MiB ASCII document retains less than 1 KiB of history, and the ASCII overwrite path avoids
unnecessary grapheme segmentation.

Delta history also has deterministic model tests for standalone edits and nested compound transactions. They compare
every forward state with an independent tokenized line model, then verify each complete undo and redo sequence. The
mix includes ASCII, CJK, emoji, combining graphemes, multiline insertion, line splitting/joining, deletion,
backspace, and overwrite while remaining below the configured history-entry limit.

Separate eviction tests exceed both history constraints. Real mixed insert, replace, and delete edits verify that the
256-entry cap discards only the oldest prefix and that every retained edit still undoes and redoes in order. A
mixed-entry queue test covers standalone and grouped deltas under a byte cap, checking exact accounting after every
push and pop and confirming that eviction retains one contiguous newest suffix.

## Overwrite paste improvement

Captured 2026-07-13 on the same host and compiler with
`cargo bench --locked --no-default-features --bench core_text -- overwrite_paste_100_kib --sample-count 10 --max-time 0.5`.
Overwriting 100 KiB at the start of a 1 MiB ASCII line measured a 1.018 ms median (100.5 MB/s).

The previous character-at-a-time path did not complete one benchmark iteration inside a 30-second measurement
window. It deleted and inserted each character separately, shifting the remainder of the long `String` twice per
character. The optimized path validates the complete operation, replaces the covered same-line ASCII range once,
extends at end-of-line when needed, and records one exact replacement delta. TUI overwrite-mode paste uses this path
for newline-free ASCII input when no prompt or overlay owns paste input. Unicode and multiline overwrite paste retain
the established characterwise behavior inside one byte-budgeted compound group.

The GUI replacement editor uses the same bulk document operation for overwrite-mode editor and clipboard paste. It
rebuilds the Iced editor mirror once after the shared edit because Iced's public editor API has no bulk arbitrary-range
replacement operation; insert-mode editor paste retains its existing per-input delta synchronization and performs no
full mirror rebuild. Both modes keep one shared undo step, including paste over an active selection.
Multi-character IME commits use that bulk path in overwrite mode after the keyboard bridge removes control characters;
single-character commits and mixed input batches retain the ordinary replacement-input path.
Replacement-editor Cut deletes the selection in the shared document directly, retaining exact undo history and
rebuilding the Iced mirror once instead of extracting adapter text into a temporary document and synchronizing it back.

A separate equal-byte-length replacement path avoids delete-then-insert behavior for ordinary character overwrite,
undo, and redo. Structural tests cover EOL extension, Unicode/multiline fallback, one-step undo/redo, search-prompt
precedence, and coalesced history storage. These local results are a forward baseline, not a portable performance
guarantee.

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
Root navigation and explicit refresh run the core sidebar's single-directory load in the same blocking worker as
recursive row construction. Create and delete flows propagate the refresh task instead of dropping it. Production
startup now constructs an empty browser placeholder and schedules that worker through Iced's initial Task, so no
directory scan is required before the first window state exists.

## GUI save preparation improvement

GUI Save and Save As no longer clone `TextDocument`, `TextBuffer`, or undo/redo histories. Preparation reuses one
editor text snapshot and records the buffer edit revision plus expected disk snapshot. A blocking worker writes that
text through the existing conflict-checked atomic adapter and returns the final `FileSnapshot`. Completion compares
revisions instead of complete strings, does not reopen the file, and performs no redundant snapshot refresh. If edits
occur during the write, the returned disk snapshot is retained and the newer buffer remains dirty.

Each GUI tile now permits at most one save worker. Repeated Save commands set one coalesced follow-up bit; completion
launches exactly one save of that tile's latest state without changing focus. Save As refuses to race an existing save
for the same tile.
