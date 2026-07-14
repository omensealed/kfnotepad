# GUI contract

This document records the current separate Iced GUI behavior. The terminal editor contract remains in
`docs/16-CLI-CONTRACT.md`.

## Commands

```bash
kfnotepad-gui --help
kfnotepad-gui --version
kfnotepad-gui --describe
kfnotepad-gui
kfnotepad-gui FILE
kfnotepad-gui FILE1 FILE2
```

- `--help`: prints GUI usage to stdout; exits 0.
- `--version`: prints `kfnotepad-gui VERSION` to stdout; exits 0.
- `--describe`: prints a deterministic non-window description for smoke checks, including GUI capability and current
  review-gap lines; exits 0.
- no arguments: opens a window with one untitled in-memory tile unless `gui_restore_last_workspace = true` is set and
  `current-workspace.v1` exists. With restore enabled, the GUI loads all workspace files that still pass the normal
  open adapter, reports skipped missing or unavailable files in status, and opens a clean untitled tile if no
  workspace files can be loaded. It does not create a file until the user opens or saves a normal file through the
  existing adapter path.
- one or more `FILE` arguments: opens each valid file as a separate tile in one window. Invalid files are reported in
  the status message; valid files still open when possible.
- unknown options such as `--unknown`: print a concise error to stderr and exit non-zero.

## File and data behavior

- GUI tiles use the same `TextDocument`, `open_text_file`, and `save_text_document` behavior as the TUI.
- Open rejects missing paths, directories, symlinks, non-regular filesystem targets such as FIFOs, sockets, and
  devices, non-UTF-8 data, and files larger than 8 MiB.
- Editing cannot grow a document beyond 8 MiB. Typed input, newline insertion, overwrite growth, and paste are rejected
  before changing text or undo history. Oversized paste-over-selection leaves both text and selection unchanged.
- Cutting or replacing a partial selection preserves an existing trailing newline. Selecting and deleting the entire
  document removes that newline as part of the selected content; undo restores the exact prior newline state.
- Save writes through the tested atomic-save adapter with symlink target rejection, existing permission preservation,
  directory and other non-regular save-target rejection, private new-file mode on Unix, and best-effort temp cleanup
  on failure.
- Save refuses to overwrite a document if its on-disk file changed or disappeared since open or the last successful
  save. This save-time check is independent of the GUI's external-change watcher and reports a conflict
  instead of silently replacing external edits.
- Saved text is normalized to LF line endings. CRLF input opens as text and writes back as LF after a save.
- Save As refuses to retarget a tile to a path already open in another tile; the already-open tile is focused or
  restored and remains the only authoritative tile for that path.
- Open GUI document tiles register their parent directories with one long-lived, debounced native watcher. Events for
  an open document trigger strong snapshot validation; clean tiles refresh in place through the same safe open path
  and enter an external-edit lock. Watcher failures visibly degrade to metadata-first polling. Directory watches are
  non-recursive and update as tile paths change.
  Locked tiles still allow scrolling and later external refreshes, but text edits are refused until the Unlock
  titlebar control is clicked. Dirty local buffers are never overwritten by external refresh; the status message reports
  the conflict for explicit user resolution.
- The GUI does not use a database, network service, account, autosave, backup file, or recent-file list. Workspace
  restore is limited to the explicit `gui_restore_last_workspace` opt-in and the deterministic
  `current-workspace.v1` project snapshot.
- Managed notes use the same normal-file storage as the TUI: Markdown files under the platform data directory
  at `.../kfnotepad/notes`. On Unix this is `$XDG_DATA_HOME/kfnotepad/notes`, or `$HOME/.local/share/kfnotepad/notes`
  when `XDG_DATA_HOME` is unset.
- Editor display preferences are shared with the TUI through `.../kfnotepad/config.toml` in the platform config directory.
  On Unix this is `$XDG_CONFIG_HOME/kfnotepad/config.toml`, or `$HOME/.config/kfnotepad/config.toml` when
  `XDG_CONFIG_HOME` is unset.
- `config.toml` persists `theme`, `syntax_theme`, `line_numbers`, `wrap`, `search_case_sensitive`,
  `gui_restore_last_workspace`, `gui_reader_mode_enabled`, `gui_reader_lines_per_minute`, `gui_font_family`,
  `gui_font_size`, and `gui_ui_font_size`. Search defaults to case-insensitive. Reader mode defaults off at
  60 lines/minute and validates speed from 20 through 240 lines/minute. GUI editor font defaults are `monospace`
  and size `16`, and GUI chrome/UI text defaults to size `14`. Older configs without the newer GUI keys continue
  to load with documented defaults.
- GUI layout persistence uses `.../kfnotepad/gui-layout.v1` in the platform config directory. On Unix this is
  `$XDG_CONFIG_HOME/kfnotepad/gui-layout.v1`, or `$HOME/.config/kfnotepad/gui-layout.v1` when `XDG_CONFIG_HOME` is unset.
- `gui-layout.v1` stores geometry only: version, browser visibility, browser width, pane split tree and ratios,
  launch/open-order leaf ordinals, and minimized ordinals. It must not contain document text, file paths, search
  queries, cursor positions, recent-file history, credentials, or unsaved buffers.
- GUI workspace/project snapshots are persisted as path-bearing files under
  `.../kfnotepad/workspaces/*.v1` in the platform config directory, or
  `$XDG_CONFIG_HOME/kfnotepad/workspaces/*.v1` when `XDG_CONFIG_HOME` is unset on Unix. Model tests exist for path
  resolution, parse/serialize, malformed snapshot rejection, layout/file-count mismatch rejection, and private atomic
  writes. Runtime project saving, listing, and opening are all wired in both current-window and new-window flows.
- Missing, malformed, unsupported, invalid-width, invalid-ratio, duplicate-ordinal, or pane-count-mismatched layouts
  are ignored and the GUI falls back to launch-time layout defaults. Old layout files without `browser_width_px`
  remain valid and use the default browser width.

## Current controls

- Header: shows dense text menu groups plus fixed-size compact icon controls for New tile, panel visibility, app
  Theme, syntax theme, Reader mode, and Save. Menu roots use minimal padding, and the adjacent 22px icon controls center their glyphs with the shared
  icon-only line-height helper and a Symbols Nerd Font Mono font preference inside the button square. Tile titlebar
  controls keep a slightly larger 24px square hit target because they sit inside pane chrome and appear on hover.
  The header does not repeat the app name, active file path, tile number, or save state; tile title bars carry the
  document identity and save status.
- Search/navigation row: Find field, case-sensitive toggle, Previous, Next, Line field, Apply line, Document start,
  and Document end. Search is case-insensitive by default; the case toggle persists through `config.toml`. Compact
  command buttons use named Nerd Font glyph constants and keep descriptive tooltips. The fields use the GUI UI
  font-size setting and app theme styling. Successful Find next/previous commands move the active editor selection
  to the match and use a stronger theme-derived selection color while the search result remains current. The GUI keeps
  an in-memory session-only Find history of the 10 most recent unique non-empty queries; clearing the Find field after
  history exists opens a small dropdown under the field, and choosing an entry restores and runs that query. Editing
  the query, entering a missing/no-match search, editing the document, or using explicit document navigation clears
  the search-highlight state.
- Menu bar: File, Edit, View, Nav, Notes, Tile, and Help command groups.
- Left panel: visible by default and switchable between Files, Workspaces, and Preferences modes. Files mode is
  rooted at the process current working directory; parent, refresh, and create-file actions use compact same-size
  icon controls with tooltips. The visible current-root path is shortened for the sidebar, while the tooltip keeps
  the full path. `..` and directories navigate, and regular files open into new tiles through existing open
  validation. Workspaces mode can save the current workspace to a deterministic
  `current-workspace.v1` snapshot, refresh/list saved projects, open a saved project in the current window, or open
  one in a fresh GUI process. Save current writes or replaces the deterministic `current-workspace.v1` snapshot used
  by opt-in auto-restore. Save named writes the current workspace under the entered project name after validating it
  through the existing project slug policy; it does not modify `current-workspace.v1`. Current-window opening
  validates every saved file path first; dirty current workspaces require clicking the same saved project a second
  time before discard. Saved-project row actions use compact icon-only New window and Delete controls with
  descriptive tooltips so narrow panels do not clip action labels. New-window opening spawns the current
  `kfnotepad-gui` executable directly and passes the saved project path through an internal launch environment value;
  the child process restores saved files and matching layout through the same project parser and file open
  validation. Delete requires clicking Delete on the same project a second time; deleting `current-workspace.v1`
  while restore is enabled shows a restore-target warning before the same explicit confirmation. Confirmed deletion
  moves the project snapshot to the operating system Trash/Recycle Bin where supported, refreshes the list, and
  rejects paths outside the configured
  workspace-project directory. Preferences mode exposes editor font family, syntax theme, editor font size, GUI UI
  font size, Line numbers, Wrap text, Case-sensitive search, Reader mode, Reader speed, and Restore last workspace
  controls backed by `config.toml`; save failures report status and roll back in-memory settings. The GUI UI
  font-size setting applies to menu roots/items, header and panel
  controls, path/project/search inputs, checkbox labels, tile title bars, minimized-tray entries, and status text;
  editor document text uses the separate editor font-size setting. Font family is a fixed preset list (`monospace`,
  `sans-serif`, `serif`,
  `jetbrains-mono`, `fira-code`) and both editor/UI font sizes are validated from 10 through 32. Missing named fonts
  fall back according to the local renderer/font stack. The panel can be hidden or restored. Its visible width can be
  adjusted with the in-panel slider and persists in `gui-layout.v1`. The panel uses snug asymmetric padding so the
  tree sits close to the left edge while keeping a small right gutter before editor content.
- Reader mode: Ctrl-R, View -> Reader mode, the header book/pause control, and the Preferences checkbox toggle
  automatic downward scrolling of the active document. The speed is configured in Preferences as lines per minute.
  Reader mode scrolls the active tile viewport without editing the document or moving the cursor, and stops itself
  at the end of the document.
- Syntax theme: Ctrl-Shift-T, View -> Syntax theme, the header syntax-theme control, and the Preferences syntax
  button cycle syntax-highlighting themes independently from the app chrome theme. Syntax theme names match the app
  presets (`nocturne`, `aurora`, `pastel`, `terminal`, `abyss`, `terror`), but foreground colors are transformed at
  the GUI syntax boundary to keep readable contrast against the selected app theme's document background.
- Restore preference: the Workspaces panel exposes a Restore last workspace checkbox for
  `gui_restore_last_workspace`. Toggling it persists through `config.toml`; save failures report status and roll back
  the in-memory setting. Enabling it also snapshots the current GUI workspace to the deterministic
  `current-workspace.v1` restore target so a later argument-free launch can restore the current files/layout without
  requiring a separate Save current action. While the preference remains enabled, later GUI file-open flows,
  replacement of the initial blank tile, new-tile creation, and explicit file-argument launches refresh the same
  deterministic snapshot so argument-free relaunches reopen the most recent workspace shape.
- Workspace auto-restore: normal GUI startup does not automatically reopen path-bearing workspace history unless
  `gui_restore_last_workspace = true` is set. With that opt-in, an argument-free launch attempts to restore
  `current-workspace.v1` through the existing project parser and file open validation. Explicit file arguments and
  explicit saved-project launches take precedence. Invalid snapshots fall back to an empty untitled tile and report
  status without writing user files. Missing or unavailable saved file paths are skipped; if no saved files can be
  loaded, an empty untitled tile is opened instead. Partially restored workspaces do not reuse saved pane geometry,
  because the saved layout may refer to files that were skipped.
- Tiles: each document is a resizable/movable pane-grid tile styled like a compact mini-window with a visible outer
  frame and a 5px default tile gap. The live editor viewport itself does not add a hover/focus border, so the
  tile frame is the primary window outline instead of a nested double-border editor box. Tile title bars show the
  document filename; normal saved state is implicit, while modified and save-failed states are shown explicitly.
  Hovering the title exposes the full path. Tile title controls use compact centered symbol glyphs with tooltips for
  Close tile, Minimize/Restore, Maximize/Restore layout, Move left/right/up/down, and Unlock when an external refresh
  lock is active. Keyboard/menu alternatives exist for core tile actions. Tile -> Equalize tiles rebuilds the current
  visible pane grid into balanced columns and rows; when the visible tile count does not fill the grid evenly, the
  remainder tile is placed in the rightmost column and uses the available height. Equalizing preserves active tile
  focus, editor text, cursor, viewport, minimized tray state, and the existing geometry-only persistence boundary.
- Minimized tray: minimized tiles leave the visible pane grid and appear in a compact tray below the menu/header.
  Restoring from the tray reinserts the tile into the pane grid without writing files or losing dirty state, file
  path, editor text, or save status. The durable layout format still stores minimized tile ordinals so saved layouts
  and workspace projects can restore the same minimized state.
- New tile: New tile button, File -> New tile, and Ctrl-N create an empty in-memory tile in the current browser
  directory using the next available `untitled.txt`/`untitled-N.txt` path. No file is created until Save.
- New tiles split the active pane along its longer side: wide panes split side-by-side, while tall panes split
  top/bottom. This follows the useful Halloy pane-grid behavior while keeping kfnotepad's fixed preset themes and
  geometry-only layout format.
- Editing: text entry uses kfnotepad's app-owned replacement editor renderer rather than Iced's native text-editor
  widget for the live document body. GUI font family and size preferences apply to editor text. Pane rendering derives
  the active editor content, resolved font, size, syntax token, highlighter theme, optional gutter snapshot, viewport
  slice, cursor, and selection metadata from a single `GuiEditorSurfaceModel` boundary. Keyboard input is routed only
  from ignored Iced key events so focused fields such as Find and path prompts keep their own text input. The live
  replacement model covers typed characters, committed IME text, Enter, Backspace, Delete, shared cursor movement,
  plain Home/End line movement, PageUp/PageDown viewport movement, Ctrl-A selection, Ctrl-Z/Ctrl-Y/Ctrl-Shift-Z
  undo/redo, mouse click/drag selection, drag-selection edge auto-scroll, Ctrl-Left/Ctrl-Right word movement, Ctrl-Backspace/Ctrl-Delete word deletion,
  Ctrl-K delete-to-line-end,
  Insert overwrite-mode toggling, Escape selection clearing, search selection, menu/keyboard clipboard operations,
  and external-edit lock refusal. Horizontal cursor movement, Backspace, Delete, and overwrite-mode replacement use
  shared core grapheme boundaries so combining marks, emoji ZWJ sequences, and regional-indicator flag pairs are not
  split by normal single-character edit commands. Word movement and word deletion also use shared grapheme-aware
  word boundaries so keyboard shortcuts do not land inside multi-codepoint characters.
  Document viewports show a thin theme-colored scrollbar when the document has more
  source lines than the visible viewport; wheel scrolling, scrollbar track clicks, and scrollbar thumb dragging
  route through the same explicit viewport-scroll path and do not dirty the document. Track/thumb scrolling does not
  move the cursor; selection edge auto-scroll moves the cursor only as the active selection focus. IME preedit/open/close events
  are non-mutating: the replacement
  renderer paints the active preedit at the cursor and requests an Iced input-method cursor rectangle for the active
  rendered visual row so platform candidate windows have a placement hint. Select-all copy/cut preserves trailing
  newlines for full-document selections. Line
  numbers and text rows render from the same viewport slice, with a painted cursor cell in the text row; the old
  temporary `Ln current/total` status strip is not shown. Legacy native text-editor scroll actions still map to the
  app-owned viewport without marking the document dirty. When the `syntax` feature is enabled, shared syntax
  highlighting is mapped into per-row colored spans in the replacement renderer, with cursor and selection overlays suppressing token color so active cells remain
  readable. Mouse click events on the replacement text body focus the pane and move the shared cursor, snapping to
  the nearest grapheme boundary for multi-codepoint characters, without
  changing the viewport when the clicked text is already visible. Mouse drag events set replacement selections
  without dirtying the document; selection copy/delete/replace expands endpoints to grapheme boundaries so partial
  multi-codepoint characters are not copied or removed. Hit testing is body-local so the line-number gutter does not skew text columns.
  Mouse-wheel and trackpad scroll events over the replacement document body scroll that pane's
  app-owned viewport without dirtying the document or moving the cursor. Keyboard/page viewport commands that
  intentionally move the cursor still keep the cursor visible. When Wrap text is enabled, the replacement renderer
  splits source lines into explicit visual rows using word-boundary wrapping with character fallback for long
  unbroken spans while preserving grapheme clusters so combining marks, emoji ZWJ sequences, and regional-indicator
  flag pairs do not split across visual rows.
  Continuation rows keep the gutter snug, show a blank line-number cell, and carry source-column offsets for cursor
  and pointer mapping. The renderer may build a larger bounded source-line slice than the logical scroll page so
  panes that grow after close/minimize can fill the newly available tile height without changing the app-owned
  viewport. That bounded slice is read from the existing `TextBuffer` line store instead of re-splitting the full
  document text during GUI view construction. Syntax highlighting for GUI tiles is cached incrementally per tile
  when the `syntax` feature is enabled:
  scrolling extends the cached syntect state from the last highlighted source line, while edits, undo/redo, save-as
  retargeting, external file refresh, workspace project restores, and tile close/reset paths invalidate or refresh
  the visible cache range. The responsive renderer only emits complete fixed-height visual rows that fit the current
  tile body, so line numbers and text do not paint partial rows below the pane frame. Active
  search results use the same selection model with a stronger search overlay in the
  replacement renderer. Richer visual regression coverage remains follow-up work. The active tile tracks
  dirty/saved/save-failed state.
- Open: File -> Open and Ctrl-O request a native file dialog through `rfd`. If native dialogs are unavailable in the
  current runtime (for example, no usable desktop session, or when `KFNOTEPAD_DISABLE_NATIVE_FILE_DIALOG` is set),
  the app falls back to `File -> Open path` and keeps the open flow in-app. Selecting a path through either path opens
  through the existing validation. If the workspace still
  contains only the initial clean empty `untitled.txt` tile, the selected file replaces that blank tile instead of
  creating a second tile. If the selected file is already open, the existing tile is focused or restored from the minimized
  tray instead of opening a duplicate tile. Otherwise, successful opens create a new tile. Canceling the dialog is a
  no-op with status feedback. File -> Open path keeps the in-app path prompt fallback. Relative fallback paths resolve
  from the current GUI file-browser directory, and fallback failures keep the prompt open and report status.
- Save: Save button, File -> Save, and Ctrl-S save the active tile only.
- Save as: File -> Save as and Ctrl-Shift-S request a native save dialog through `rfd`. If native save dialogs are
  unavailable in the current runtime, the app falls back to `File -> Save as path` and keeps the in-app prompt flow.
  Selecting a path retargets the active tile through the existing atomic save adapter; canceling the dialog is a
  no-op with status feedback. File -> Save as path keeps the in-app path prompt fallback. Relative fallback paths
  resolve from the current GUI file-browser directory, and fallback failures keep the original tile path and prompt open.
- Managed notes: Notes -> Open note shows an in-app note-title prompt. Successful note titles create or open the
  matching managed Markdown file through the existing managed-notes adapter and open it in a new tile. Notes -> List
  notes shows existing managed notes in deterministic filename order; clicking one opens it in a new tile. Each
  listed note also exposes a Delete control that requires clicking Delete on the same note a second time before
  removal. Confirmed deletion moves the note to the operating system Trash/Recycle Bin where supported and is limited
  to visible direct `.md` files inside the configured managed-notes directory; outside paths, non-note targets,
  symlinks, and directories are rejected by the shared adapter. If a note is already open in any tile, GUI deletion is
  refused until that tile is closed so a later Save cannot silently recreate the removed note. After successful
  deletion, the notes list refreshes.
- Close tile: title close button, File -> Close tile, or Ctrl-F4 close the active tile. Closing a dirty tile requires a
  second close request before discarding unsaved changes. Closing the last remaining tile does not leave the
  workspace empty; after any required dirty confirmation it resets that pane to a clean in-memory `untitled.txt` tile
  without writing a file.
- Application close: a clean workspace closes immediately. File -> Quit, Ctrl-Q, and the window close button use the
  same dirty-window confirmation path. A dirty workspace requires a second close request before discarding unsaved
  changes.
- Clipboard/Edit: Edit -> Undo, Redo, Copy, Cut, Paste, Select all, Ctrl-Z, Ctrl-Y, Ctrl-Shift-Z, Ctrl-C, Ctrl-X,
  Ctrl-V, and Ctrl-A operate on the active
  replacement editor selection through kfnotepad's editor model and Iced's public clipboard tasks. Copy/Cut require a
  selection; Cut and Paste update the in-memory document and dirty state but do not save files.
- Search: active-pane search field, Next/Prev buttons, Edit menu commands, Ctrl-F, F3, and Shift-F3. Successful
  search moves to the match and selects the matched text using the app-owned editor selection highlight. If a query
  matches inside a multi-codepoint grapheme cluster, the match selection expands to the whole grapheme so the editor
  never selects only part of one visual character.
- Navigation: Document start/end buttons, Nav menu commands, Ctrl-Home, Ctrl-End, line-number field, Apply line
  button, and Ctrl-G.
- Theme: Theme button, View -> Theme, and Ctrl-T cycle the fixed preset themes shared with the TUI:
  `nocturne`, `aurora`, `pastel`, `terminal`, `abyss`, and `terror`. Older config files that still say `paper`
  load as `pastel` for compatibility. Pastel keeps syntax text readable on its light background by darkening pale
  highlighter foregrounds at render time.
- Syntax theme controls cycle independently when the `syntax` feature is enabled. A lean
  `--no-default-features --features gui` build renders plain text and reports that syntax highlighting is unavailable;
  native release packages use `--features "gui syntax"`.
- Help: Help -> Open help opens a built-in `kfnotepad-help.md` Markdown tile with user-facing guidance for files,
  tiles, editing, search/navigation, reader mode, app/syntax themes, workspaces, preferences, managed notes,
  external file changes, and save/open safety rules. Opening Help again focuses the existing help tile instead of
  creating duplicates. `kfnotepad-gui --help` stays as concise terminal usage output and lists the current headline
  shortcuts including reader mode, syntax theme, and the default case-insensitive search behavior.
- Menus: File/Edit/View/Nav/Notes/Tile/Help use `iced_aw::MenuBar` roots with `iced_aw::menu::Item::with_menu` dropdowns
  anchored to compact header items instead of manual x/y-positioned overlays or inserted toolbar rows. Command rows
  are regular widgets inside `iced_aw::menu::Item::new`, matching the crate examples. The current shallow command
  groups intentionally stay flat until a menu group gains enough depth to justify nested hover submenus. Menu actions
  route through the same command path as their direct buttons/shortcuts.
- Preferences: the left-panel Preferences mode persists shared Line numbers and Wrap text settings plus GUI editor
  font family, editor text size, and separate GUI UI/chrome text size. Line numbers toggles the editor gutter, and
  Wrap text affects editor wrapping immediately after the setting changes.
- File browser toggle: Files button, View -> Files, and Ctrl-B. When the left panel is hidden, no side strip remains;
  the editor/search/pane-grid area expands across the freed width.
- Files panel: Files mode renders a local recursive directory tree rooted at the current browser directory. The tree
  labels and icons use the GUI UI/chrome font-size preference. The tree may expand/collapse directories. A single
  row click selects a file or directory; double-clicking a file opens it through the same safe adapter path as other
  GUI open actions, and double-clicking a directory makes it the tree's new root and updates the current browser
  directory for relative prompts. The explicit parent-directory control resets the tree root upward. Refresh reloads
  the current root so external filesystem changes become visible. Create file and Create folder prompt for a name and
  create under the selected directory when a directory is selected, otherwise under the current browser directory.
  Create file writes an empty file through the existing safe save path, refreshes the tree, and opens the new file
  through the existing safe open path. Create folder uses a single-directory create and refreshes the tree. Names
  must be relative UTF-8 child paths without parent traversal. Delete selected requires a second click on the same
  selection; confirmed deletion moves the selected path to the operating system Trash/Recycle Bin, directory deletion
  warns that all subdirectories and files move with it, refuses symlinks, and
  refuses files/directories that are open in editor tiles.
- Minimize/restore: tile title control, Tile -> Minimize, and Ctrl-M minimize the active tile into the tray. Tray
  restore buttons restore minimized tiles. The last visible tile cannot be minimized.
- Maximize/restore layout: tile title control, Tile -> Maximize, and Ctrl-Shift-M. Maximizing is transient view
  state; it does not change or persist split geometry.
- Move tile: hover title controls, Tile menu commands, Ctrl-Shift-arrow, and Iced pane drag/drop.
- Equalize tiles: Tile -> Equalize tiles rearranges visible panes into a balanced grid while leaving minimized tiles
  in the minimized tray.
- Resize tile: drag pane-grid split handles.

## Accessibility status

- Core GUI actions have descriptive visible labels and deterministic tests checking they keep keyboard or menu access.
- Prominent header, menu, left-panel, search/navigation, workspace-project, and tile-title controls use hover
  tooltips while keeping visible text or menu/keyboard alternatives. Header action buttons, left-panel mode
  selectors, and compact search/navigation buttons may render as icon-only controls because their tooltips, menus,
  fields, and shortcuts keep the descriptive command text available.
- Tooltips render with an opaque dark background, readable text, and a border so they remain legible over editor
  content and themed chrome.
- Hover title controls are not the only path for save, close, search, navigation, file-browser toggle, theme,
  minimize, or movement commands.
- Deterministic focus-order inventory coverage records the intended visible traversal order for menu/header,
  file-browser, search/navigation, tile controls, and editor states, including hidden-browser and minimized-tile
  variants.
- Dirty state and save failures are reported as text, not color alone.
- A local screenshot/pixel smoke exists for nonblank-window coverage. It launches with disposable files/config,
  matches the specific `kfnotepad-gui` window title for that temporary fixture, captures that window with `maim` or
  ImageMagick `import`, and checks dimensions/color variance. A full assistive-technology audit, high-contrast
  review in a live window, and rich visual regression suite remain future work.

## Current gaps

- The GUI is published alongside the TUI as an alpha front end; it does not replace the terminal editor.
- Native Open/Save as dialogs are implemented with path-prompt fallback, but exhaustive desktop-environment dialog
  smoke coverage is still manual. No command palette, recent-file list, print/export, or live assistive-technology
  audit has been completed. Workspace restore and app-owned drag selection are implemented and covered by state tests,
  but their complete platform/desktop-environment matrix remains manual.
- A local screenshot/pixel smoke exists, but rich CI visual regression and exact layout assertions are not part of
  the default suite yet.

## Manual GUI smoke

Use disposable files and config, not personal data:

```bash
repo=$(pwd)
tmpdir=$(mktemp -d)
printf 'first\n' > "$tmpdir/first.txt"
printf 'second\n' > "$tmpdir/second.rs"
XDG_CONFIG_HOME="$tmpdir/config" "$repo/target/release/kfnotepad-gui" "$tmpdir/first.txt" "$tmpdir/second.rs"
rm -rf "$tmpdir"
```

Verify:

- The window opens with two document tiles.
- The file browser can be hidden/restored, can expand directories in the tree, and can reset the root upward with the
  parent-directory control.
- The file browser Refresh control picks up a disposable file created outside the GUI.
- The file browser Create file/Create folder controls create under the selected directory when one is selected.
- A regular file single-click selects only; double-click opens a new tile.
- Delete selected requires confirmation and moves selected files/directories to the operating system Trash/Recycle Bin;
  selected directories move recursively only after warning.
- File -> Open can open a disposable relative path into a new tile.
- The active tile can be edited, saved, and closed.
- File -> Save as can save the active tile to a disposable relative path without changing the original file on
  failure.
- Notes -> Open note can create/open a disposable managed note when `XDG_DATA_HOME` points at the temporary
  directory.
- Notes -> List notes shows disposable managed notes and opens the selected note in a new tile.
- A dirty tile requires a second close request before discard.
- A dirty application window requires a second close request before discard.
- Theme cycling updates the GUI and writes only the shared config file under the disposable config directory.
- Preferences can change editor font family, editor font size, and UI font size, and the active editor/chrome text
  updates without writing outside the disposable config directory.
- Preferences can toggle Line numbers and Wrap text, and the editor body updates without writing outside the
  disposable config directory.
- Edit -> Select all, Copy, Cut, and Paste operate on the active editor selection. Cut/Paste change only the
  in-memory dirty tile until Save is used.
- Moving, dragging, resizing, minimizing, and restoring tiles changes layout.
- After closing and reopening with the same number of launch files and disposable `XDG_CONFIG_HOME`, the saved layout
  geometry is restored.
- Deleting `"$tmpdir/config/kfnotepad/gui-layout.v1"` and relaunching returns to the default launch-time layout.

Bounded launch smoke for automated/local checks:

```bash
tmpdir=$(mktemp -d)
printf 'first\n' > "$tmpdir/first.txt"
printf 'second\n' > "$tmpdir/second.txt"
XDG_CONFIG_HOME="$tmpdir/config" timeout 5s cargo run --locked --no-default-features --features "gui syntax" --bin kfnotepad-gui -- "$tmpdir/first.txt" "$tmpdir/second.txt"
status=$?
rm -rf "$tmpdir"
test "$status" -eq 124
```

Status 124 means the window launched and stayed open until the timeout, which is expected for this bounded smoke.

Local screenshot/pixel smoke for X11 sessions:

```bash
./scripts/gui-visual-smoke.sh
```

This builds `kfnotepad-gui`, launches it with disposable files and config, captures the newly launched X11 window
with `xprop` and ImageMagick `import`, writes the ignored screenshot
`target/gui-visual-smoke/kfnotepad-gui.png` unless `KFNOTEPAD_GUI_SMOKE_SCREENSHOT` is set, and fails if the image is
too small, too low-color, or visually blank. The disposable fixture enables line numbers and wrap, opens a long
Markdown document plus a Rust file, and therefore exercises the wrapped-gutter path that has historically regressed.
It verifies rendered output for that path, but not exact pixel layout correctness or live accessibility behavior.

## Package and rollback expectations

- `./scripts/package.sh` stages both `bin/kfnotepad` and `bin/kfnotepad-gui` in the local tarball.
- Tagged releases publish Linux, Windows, and macOS alpha packages through the GitHub release workflow. Linux remains
  the primary support tier; Windows packages are unsigned, while macOS packages are ad-hoc signed but not notarized.
- Removing the GUI from a local install is deleting `bin/kfnotepad-gui`; the TUI binary and user notes continue to
  work.
- Reset GUI layout by deleting `gui-layout.v1`.
- Reset saved GUI workspace/project history by deleting `workspaces/` under the same config directory.
- Reset shared display preferences and GUI font preferences by deleting or editing `config.toml`.
- Uninstall by removing the user-owned install prefix documented in `docs/13-OPERATIONS.md`.
