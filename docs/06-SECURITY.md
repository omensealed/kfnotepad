# Security and privacy

## Initial risk profile

- Network access: No
- User accounts: No
- Personal data: Yes
- Payments: No
- Additional notes: do not enter credentials

Threat model status: current as of 2026-07-08 for the arbitrary-file terminal/editor GUI journey.

## Assets

- User-authored note/file contents, including possible personal data or pasted secrets, across all open in-memory
  tabs and GUI tiles.
- Existing file metadata relevant to safe replacement, especially permissions and symlink/directory state.
- Terminal state: raw mode, alternate screen, cursor visibility, and user prompt restoration after exit or error.
- GUI display preferences and geometry-only layout files under XDG config.
- Repository source, tests, docs, and lockfile integrity.
- User trust in the CLI contract: `kfnotepad FILE` must not unexpectedly write, leak, or hide data loss.

## Actors

- Primary user editing files they own.
- Local processes or users that can race file paths, symlinks, permissions, or directories in the same filesystem.
- Malicious or malformed files opened intentionally or by mistake.
- Contributors reading repository text, generated docs, or external advisory data.

## Entry Points And Trust Boundaries

- CLI arguments are untrusted input.
- File paths, file metadata, file contents, symlinks, directories, and parent directories are untrusted filesystem input.
- Terminal key events, mouse events, tab/close/sidebar commands, and terminal size are untrusted terminal input.
- GUI window events, mouse/keyboard input, pane-grid resize/drag/drop events, file-browser clicks, config files, and
  layout files are untrusted local input.
- Repository docs, generated recommendations, and external text are untrusted data unless consistent with maintainer
  intent and project policy.
- No runtime network, database, user account, credential, or external service boundary exists in the current app.

## Sensitive Data Inventory

- User file contents may contain personal data or secrets.
- Search queries, open tab buffers, and in-memory undo snapshots may contain user file content.
- GUI open tile buffers, search queries, and Iced editor content may contain user file content.
- Error messages may include paths and OS errors, but must not include file contents.
- `config.toml` stores only display preferences. `gui-layout.v1` stores only pane geometry, browser visibility,
  browser width, and minimized ordinals; it must not store document text, file paths, cursor positions, search
  queries, recent-file history, credentials, or unsaved buffers.
- Tests use synthetic temporary files only; no production or personal fixtures are required.

## Abuse Cases And Current Controls

- **Oversized file memory pressure:** open and save reject content over 8 MiB before reading/writing. Covered by
  sparse-file open test and oversized-save test.
- **Directory and non-regular file overwrite attempt:** open/save reject directory targets, symlinks, and other
  non-regular filesystem targets such as FIFOs, sockets, and devices. Covered by
  `rejects_directory_save_target_without_temp_file`, `rejects_fifo_without_reading_from_it`, and
  `rejects_fifo_save_target_without_temp_file`.
- **Symlink target surprise:** open and save reject symlink paths rather than following them. Covered on Unix by
  `rejects_symlink` and `rejects_symlink_save_target`.
- **Missing parent or temp creation failure:** save fails before target creation when the temp sibling cannot be
  created. Covered by `missing_parent_directory_fails_before_target_creation`.
- **Accidental overwrite/data loss:** save writes a temporary sibling file, flushes, then renames; existing
  permissions are preserved where possible. Expected validation and temp-creation failures leave the original target
  untouched. Save also compares the current disk snapshot with the document snapshot captured on open or last
  successful save; if the file changed or disappeared, save is refused with an explicit conflict message rather than
  silently replacing external edits. Covered by shared, TUI, and GUI save-conflict tests.
- **Line-ending ambiguity:** saved document text is normalized to LF. CRLF input is accepted as UTF-8 text and writes
  back as LF after save; trailing newline and multiple-trailing-newline behavior is covered by save-adapter tests.
- **Accidental dirty quit or tab close:** interactive Ctrl-Q requires confirmation for dirty buffers, and Ctrl-F4
  requires a separate confirmation before closing a dirty tab. Plain sidebar Enter is blocked while the current
  document is dirty; sidebar Ctrl-Enter opens a new tab through the same file-open validation without discarding the
  dirty active document. Covered by event-handler and workspace tests.
- **Accidental dirty GUI tile close or app quit:** GUI tile close and application close require a second request when
  unsaved content would be discarded. Covered by GUI binary tests.
- **Sensitive metadata in GUI layout:** runtime layout persistence stores ordinal geometry only. Shared layout tests
  and GUI runtime tests assert path/content omission and safe fallback for malformed or incompatible files.
- **Terminal left in raw mode:** a drop guard restores cursor, alternate screen, and raw mode on normal loop exit and
  errors propagated through the loop. Unit coverage verifies the terminal backend restore hook runs on drop, and
  setup now disables raw mode if alternate-screen entry fails after raw mode was enabled. Full end-to-end
  pseudo-terminal coverage is still deferred.
- **Secret leakage through logs:** the app does not currently log. User-facing errors include operation/path context
  but not file contents. Save-failure status coverage verifies buffer contents are not included in the failure
  message.
- **Untrusted generated or external text:** project policy treats repository-adjacent generated text and external
  advisory text as untrusted unless aligned with maintainer intent and the task at hand.

## Remaining Phase 2 Risks

- Open and save perform metadata validation before the later read or rename operation. A same-path filesystem race
  remains possible between validation and use; closing that gap may require Unix-specific no-follow/open-by-handle
  behavior.
- Atomic rename semantics vary by filesystem; tests cover normal local temporary directories only.
- Temp-file cleanup on all failure modes is best-effort and not exhaustively fault-injected.
- Workspace snapshots store local file paths only. Restoring a stale snapshot skips missing or unavailable files and
  reports status; it does not recreate, truncate, or overwrite paths that no longer load. If no files can be loaded,
  the editor opens a clean untitled document.
- Undo history is bounded by byte budget and count to reduce memory growth while editing within the 8 MiB file limit.
  The current limit is 64 MiB and uses full-delta snapshots plus snapshot coalescing around typed insert runs.
- End-to-end raw-mode cleanup is still manually verified; unit coverage proves the drop path calls restore but does
  not exercise a real pseudo-terminal.
- No automatic backup/restore feature exists. Recovery depends on the original file remaining untouched for expected
  save failures and on user-managed file backups after a successful overwrite.

## Baseline controls

- Project crate code is now compiled with `#![forbid(unsafe_code)]` in binary and library roots to prevent accidental
  `unsafe` introduction without review.
- Deny by default at permission and authorization boundaries.
- Validate type, size, range, format, path, and ownership of untrusted input.
- Parameterize database queries and escape/encode output for its context.
- Use safe path joins and prevent traversal, symlink surprises, and unsafe archive extraction.
- Apply timeouts, response-size limits, and explicit redirect/TLS policy to network calls.
- Store passwords with a modern password-hashing implementation supplied by a reviewed library.
- Keep secrets outside source and logs; rotate any secret accidentally exposed to the repository or release artifacts.
- Use least-privilege service/database accounts and separate development from production.
- Pin dependencies and review install/build scripts before executing them.
- Back up before destructive migrations and test restore, not only backup creation.

## Release security gate

- Threat model and sensitive-data inventory current.
- No secrets or production data in Git history, artifacts, logs, or fixtures.
- Authentication/authorization and abuse-case tests pass where applicable.
- Dependency and license review complete; lockfiles current.
- Error messages/logs do not disclose secrets or unnecessary personal data.
- Backup, migration, rollback, and incident steps documented and exercised where relevant.
- Security check scripts (`scripts/security-check.sh`) run as part of `scripts/check.sh`; the step executes dependency policy and
  advisory scans when tooling is installed.
