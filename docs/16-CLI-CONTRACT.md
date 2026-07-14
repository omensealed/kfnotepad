# CLI and keybinding contract

This document records the current public behavior. Update it when CLI arguments, exit codes, keybindings,
file format behavior, or terminal behavior changes.

## Commands

```bash
kfnotepad --help
kfnotepad --version
kfnotepad
kfnotepad FILE
kfnotepad --note TITLE
kfnotepad --notes
```

- `--help`: prints usage, current behavior, and editor keys to stdout; exits 0.
- `--version`: prints `kfnotepad VERSION` to stdout; exits 0.
- no arguments: prints a launch-ready message to stdout and exits 0 unless `gui_restore_last_workspace = true` is
  set and a TUI `current-workspace.v1` exists. With that explicit restore preference in an interactive terminal,
  the TUI restores loadable files from that workspace through the same project parser and file-open validation used
  by the GUI. Missing or unavailable workspace files are skipped with a status message; if no workspace files can be
  loaded, the TUI starts with a clean `untitled.txt` tab.
- `FILE` with interactive stdin and stdout: opens the terminal editor for a regular, non-symlink UTF-8 file up to
  8 MiB.
- `FILE` in a non-interactive context: prints a read-only file summary to stdout; exits 0.
- `--note TITLE` with interactive stdin and stdout: creates or opens a managed Markdown note by safe title/slug and
  opens it in the terminal editor.
- `--note TITLE` in a non-interactive context: creates or opens the managed note and prints a summary to stdout;
  exits 0.
- `--notes`: lists managed note filenames to stdout in deterministic filename order; exits 0.
- unknown option or invalid argument shape: prints a concise error to stderr; exits 2.
- open/read validation failures: print a concise error to stderr; exit 2.
- terminal runtime failures: print a terminal error to stderr; exit 1.

## File behavior

- Notes are normal files on disk; there is no database.
- Managed notes are stored as normal `.md` files under the platform data directory at `.../kfnotepad/notes`. On Unix this is
  `$XDG_DATA_HOME/kfnotepad/notes`, otherwise `$HOME/.local/share/kfnotepad/notes`.
- Managed note titles are local names, not paths. Empty names, hidden names, `.`/`..`, traversal, path separators,
  control characters, and names that normalize to no usable slug are rejected.
- Managed-note listing includes only direct regular visible `.md` files. It ignores directories, hidden files,
  non-note extensions, non-UTF-8 names, and symlinks. A missing managed-notes directory lists as empty.
- Open rejects missing paths, directories, symlinks, non-regular filesystem targets such as FIFOs, sockets, and
  devices, non-UTF-8 data, and files larger than 8 MiB. The actual opened-file read is bounded to 8 MiB plus one
  sentinel byte, so growth after metadata inspection is still rejected safely.
- Editing cannot grow a document beyond 8 MiB. Typed input, newline insertion, overwrite growth, and paste are rejected
  before changing text or undo history; the status line reports the limit.
- Save writes through the tested adapter: temporary sibling file, flush, atomic rename, symlink save-target rejection,
  directory and other non-regular save-target rejection, existing permission preservation, `0o600` new-file mode on
  Unix, and best-effort temp cleanup on failure.
- Save refuses to overwrite a document if its on-disk file changed or disappeared since open or the last successful
  save. Save-conflict snapshots use the same bounded read; an externally grown target above 8 MiB is left untouched,
  no temporary file is created, and the in-memory document remains dirty. The first alpha behavior is a clear
  conflict message rather than merge UI.
- Saved text is normalized to LF line endings. CRLF input opens as text and writes back as LF after a save.
- The editor does not create automatic backup files; restore successful overwrites from normal filesystem backups,
  snapshots, or version control.

## Editor keys

- Arrow keys: move cursor. Horizontal movement steps over whole user-perceived characters/grapheme clusters.
- Ctrl-Left/Ctrl-Right: move cursor by word, crossing line boundaries when needed. Word movement uses grapheme
  cluster boundaries for multi-codepoint characters.
- PageUp/PageDown: move by one visible editor page.
- F2: open the command palette. While the palette is open, type part of a command label, shortcut, or menu group to
  filter commands; Up/Down/PageUp/PageDown/Home/End move the selection, Enter runs the selected command through the
  same path as the F10 menu, and Esc closes the palette.
- F10: open or close the keyboard menu. While the menu is open, Left/Right or Tab/Shift-Tab switch
  File/Edit/View/Go/Tabs/Help groups, Up/Down choose a menu item, Home/End choose the first/last item, Enter runs the
  selected command, and Esc closes the menu. The File menu includes new-file, save, files, and quit commands. The
  Edit menu includes find, undo, redo, word deletion, and repeat-search commands. The Go menu includes page,
  document-edge, line, and word movement commands. The Tabs menu includes previous-tab, next-tab, and close-tab commands. The Workspace menu
  saves the current tab set, saves a named workspace, lists saved projects, opens or deletes a named project, opens
  the current workspace snapshot, and toggles restore-last-workspace. The Help menu opens a maintained built-in help
  document and keeps a compact shortcut reference. Dropdown items show shortcut hints where a direct keybinding
  exists.
- Home: move to the start of the current line.
- End: move to the end of the current line.
- Ctrl-A/Ctrl-E: move to the start/end of the current line.
- Ctrl-Home: move to the start of the document.
- Ctrl-End: move to the end of the document.
- Printable characters: insert at cursor.
- Ctrl-N: create a new untitled file tab outside the file sidebar. The tab receives the next available
  `untitled.txt`/`untitled-N.txt` path in the current sidebar directory when the sidebar is open, otherwise in the
  process current working directory. No file is created until Ctrl-S/File -> Save.
- Tab: insert one indentation level as four spaces at the cursor.
- Shift-Tab: remove up to one four-space indentation level immediately before the cursor.
- Enter: split the current line.
- Backspace: delete the previous user-perceived character/grapheme cluster before the cursor; joins with previous line at line start.
- Delete: delete the user-perceived character/grapheme cluster at the cursor; joins with next line at line end.
- Ctrl-Backspace: delete from the previous word boundary to the cursor, crossing line boundaries when needed.
- Ctrl-Delete: delete from the cursor through the current or next word, crossing line boundaries when needed.
  Word deletion uses grapheme cluster boundaries for multi-codepoint characters.
- Ctrl-K: delete from the cursor to the end of the current line. At line end it is a no-op.
- Insert: toggle overwrite mode. While overwrite mode is on, printable characters replace the character under the
  cursor; at line end they insert normally without joining the next line. Press Insert again to return to normal
  insert mode.
- Ctrl-Z: undo recent edits since the last save. Undo history is bounded to avoid unbounded memory growth.
- Ctrl-Y: redo the last undone edit. A new edit after undo clears the redo stack.
- Ctrl-F: search text; the active search prompt appears in the status line, Enter accepts the query, and Esc cancels.
  Accepted search matches are highlighted in the visible document body. Search defaults to case-insensitive matching.
  If a query matches inside a multi-codepoint grapheme cluster, the cursor/highlight expands to the whole grapheme so
  search never leaves the editor positioned inside one visual character.
- Ctrl-Shift-F: toggle exact-case search. The setting is persisted with the rest of the editor preferences.
- Up/Down while the search prompt is active: recall the last ten unique non-empty accepted search queries for the
  current session.
- F3: repeat the last non-empty search forward from after the current cursor position, wrapping to the first match
  when needed and honoring the current exact-case setting. If no search has been accepted, F3 reports that no
  previous search exists.
- Shift-F3: repeat the last non-empty search backward from before the current cursor position, wrapping to the last
  match when needed and honoring the current exact-case setting. If no search has been accepted, Shift-F3 reports
  that no previous search exists.
- Ctrl-G: go to a one-based line number; the active line prompt appears in the status line, Enter accepts the line,
  and Esc cancels. Empty, invalid, zero, and out-of-range lines do not move the cursor.
- Ctrl-B: toggle the file sidebar. While the sidebar is open, Up/Down changes the selected entry, mouse wheel over
  the sidebar moves selection without wrapping, Enter activates it, and Esc closes the sidebar. Directory entries
  navigate. Regular file entries open or focus a visible tab instead of replacing the active tab. Reopening the
  sidebar returns to the last visited sidebar directory for the current editor session.
- While the file sidebar is open, Ctrl-N prompts for a local child filename, Ctrl-D prompts for a local child
  directory name, and Delete prompts for deletion confirmation. Create prompts reject empty names, `.`/`..`, hidden
  names, path separators, and control characters. Confirmed file/directory deletion moves the selected path to the
  operating system Trash/Recycle Bin. Directory deletion requires typing `yes` after a warning that all contents move
  with it. Symlink deletion is refused. Deleting a file that is open and modified in any editor tab is refused.
- Ctrl-L: toggle line numbers for the current editor session.
- Ctrl-T: cycle built-in themes for the current editor session: `nocturne`, `aurora`, `pastel`, `terminal`,
  `abyss`, and `terror`.
- Ctrl-Shift-T: cycle syntax highlighting themes independently from the chrome theme, using the same preset names
  and persisted `syntax_theme` key as the GUI.
  Builds made with `--no-default-features --features tui` omit the syntax engine, render all files as plain text,
  and report that syntax highlighting is unavailable when this action is requested. Default and release builds
  include the `syntax` feature.
- Ctrl-R: toggle reader mode for automatic downward viewport scrolling. Reader mode uses the persisted
  `gui_reader_lines_per_minute` speed, does not move the edit cursor, stops at the end of the document, and stops
  when the user edits or switches files/tabs. While reader mode is active, the viewport is clamped only to document
  bounds and does not snap back to the edit cursor; if the edit cursor scrolls offscreen, the terminal cursor is
  hidden until the cursor row becomes visible again.
- Ctrl-W: toggle word wrapping for the current editor session.
- Mouse left-click: move the cursor in the visible editor body, including wrapped visual rows, snapping to the
  nearest grapheme boundary for multi-codepoint characters; select visible tab labels, open top-menu groups, and
  choose visible dropdown menu items. The highlighted terminal cursor cell and
  subsequent typed input target the same document row/column.
- Mouse wheel over the editor body: move the cursor three rows up/down and scroll the viewport as needed. Wheel
  scrolling is ignored while menus or prompts are active.
- Ctrl-S: save.
- Ctrl-Q: quit globally, including while prompts, menus, or the sidebar are active; dirty buffers require Ctrl-Q
  twice.
- Ctrl-PageUp/Ctrl-PageDown: switch to the previous/next editor tab. With only one tab open, these report that only
  one tab is open.
- Ctrl-F4: close the active editor tab. Closing the only tab is refused; closing a dirty tab requires Ctrl-F4 twice
  and discards unsaved changes in that tab only.
- Terminal and window-manager interception caveat: kfnotepad can only handle key events delivered by the terminal
  emulator. If a desktop environment, terminal, multiplexer, or shell intercepts a chord first, use F2 command
  palette, the F10 menu, mouse/menu path, or a future configurable keymap instead of relying on that chord.

## Editor preferences

- Editor display preferences persist in `.../kfnotepad/config.toml` under the platform config directory. On Unix this is
  `$XDG_CONFIG_HOME/kfnotepad/config.toml`, otherwise `$HOME/.config/kfnotepad/config.toml`.
- Persisted keys are `theme`, `syntax_theme`, `line_numbers`, `wrap`, `search_case_sensitive`,
  `gui_restore_last_workspace`, `gui_reader_mode_enabled`, and `gui_reader_lines_per_minute`.
- `gui_restore_last_workspace` is a shared opt-in restore preference used by the GUI and TUI. The name remains for
  config compatibility. In the TUI, enabling it allows argument-free interactive launch to restore the deterministic
  `workspaces/tui/current-workspace.v1` snapshot when present.
- `gui_reader_mode_enabled` and `gui_reader_lines_per_minute` are shared reader-mode preferences used by the GUI and
  TUI. The names remain for config compatibility. In the TUI, Ctrl-R and View -> Reader mode toggle the enabled
  value.
- Unknown config keys are ignored. Malformed known values fall back to safe defaults. Config writes use an atomic
  temporary file and private Unix permissions.

## Deferred behavior

- Note tags/folders/search/sync/import/export, in-app clipboard integration, drag selection, context menus, and
  automatic backup files are not part of the current contract.
- Rendering uses ANSI color for a header, status bar, command bar, optional line-number gutter, cursor-following
  vertical scrolling, line/column status, and, when the `syntax` feature is enabled, extension-based syntax
  highlighting with plain fallback. Screen rows are
  positioned explicitly. Long paths in the header are shortened from the left so the saved/modified state remains
  visible. Long lines scroll horizontally by default as the cursor moves; Ctrl-W toggles runtime word wrapping. The
  status line reserves critical metadata using compact labels: `num:on/off`, `wrap:on/off`, `x:N`, theme, and
  saved/modified state. The editor keeps the terminal cursor visible at the current editor position and paints the
  active text cell with reverse video as a second cursor-location affordance. Body rendering uses terminal display
  width for clipping, wrapping, horizontal scrolling, and cursor placement. Wrapping preserves grapheme clusters so
  combining marks, emoji ZWJ sequences, and regional-indicator flag pairs do not split across visual rows; tabs render as spaces using four-column
  tab stops. Word wrap prefers whitespace boundaries and falls back to character wrapping only for a single word that
  exceeds the visible width. F2 opens a centered keyboard command palette backed by the same command dispatcher as
  the menu. F10 opens a keyboard-driven top menu/drop-down for File, Edit, View, Go, Tabs,
  Workspace, and Help
  commands; the menu bar is hidden in narrow headers when needed to preserve saved/modified state. The bottom footer
  is a compact bounded hint for menu/help, save, files, and quit; detailed usage lives in the built-in help document
  opened from the Help menu instead of the footer. During active search and go-to-line prompts, the terminal cursor moves to the status-line
  prompt and long queries keep their visible tail when narrowed. On terminals that report keyboard-enhancement support, the editor requests modifier
  disambiguation for the session so controls such as Ctrl-Backspace can arrive as distinct key events; terminals
  without support keep the normal input path. The editor enables mouse capture only during the interactive editor
  session and restores it during terminal cleanup. It does not write terminal clipboard sequences.

## Tabbed-document behavior

- Each open tab is backed by the same `TextDocument` model and open/save adapters as the one-file editor path.
- The launch file starts as the first tab. Sidebar `Enter` opens the selected regular file as a new active tab after
  the same UTF-8, symlink, directory, and size validation used by `kfnotepad FILE`. If that file is already open,
  the existing tab is focused instead of duplicated. `Ctrl-Enter` keeps the same open/focus behavior when the
  terminal delivers it, but the normal path does not depend on that modifier chord.
- Dirty-tab close confirmation and dirty-editor quit confirmation are separate. The second Ctrl-F4 closes only the
  active dirty tab without saving it; the second Ctrl-Q exits the editor and discards unsaved changes in any dirty
  open tabs.
- The tab strip appears only when more than one tab is open. Active and dirty state are visible in the strip. The
  strip wraps across additional rows when tab labels exceed the terminal width; editor body and mouse hit testing
  start below the wrapped rows. Visible tab labels can be clicked with the mouse, and tab commands are available from
  direct keybindings and F10 -> Tabs.

## TUI workspace-project behavior

- TUI workspace projects use the same path-only `*.v1` project format as the GUI, but live under a separate TUI
  namespace under the platform config directory as `.../kfnotepad/workspaces/tui`. On Unix this is
  `$XDG_CONFIG_HOME/kfnotepad/workspaces/tui`, or `$HOME/.config/kfnotepad/workspaces/tui` when `XDG_CONFIG_HOME` is unset.
- TUI saves write path-only projects with no GUI layout geometry. GUI-created projects with layout data are accepted;
  the TUI opens their file paths and active ordinal while ignoring tile geometry, browser width, and minimized state.
- Workspace -> Save current writes or replaces the deterministic `current-workspace.v1` snapshot.
- Workspace -> Save named prompts for a project name and writes a slugged project file through the existing
  `gui_workspace_project_path` policy. Up/Down in the prompt cycles existing saved project names so they can be
  overwritten without retyping.
- Workspace -> Manage projects opens a visible workspace-manager panel. Up/Down/PageUp/PageDown/Home/End move the
  selection; Enter opens the selected project; `S` saves the current tab set over the selected project; `D` or Delete
  starts the typed `yes` delete confirmation; `N` starts the save-named prompt; Esc closes the panel.
- Workspace -> Open project prompts for a project name, loads each file through the normal open adapter, reports
  missing or unavailable files in the status bar, and replaces the current tab set with the files that can be
  opened. If no project files can be loaded, it opens a clean `untitled.txt` tab instead. Up/Down in the prompt
  cycles saved project names.
- Workspace -> Delete project prompts for a saved project name, supports Up/Down cycling, and moves only the saved
  workspace project snapshot to the operating system Trash/Recycle Bin after typed `yes` confirmation.
- Opening a workspace while any current tab is modified requires typing `yes` in a confirmation prompt. Canceling or
  failing to read the project snapshot leaves the current tabs unchanged.
- Workspace -> Restore last toggles the shared `gui_restore_last_workspace` preference. When enabled, TUI launch and
  later TUI tab open/focus/switch/close flows refresh the TUI `current-workspace.v1` snapshot. An argument-free
  interactive `kfnotepad` launch attempts to restore that TUI snapshot; explicit file and note launches keep their
  existing behavior but also refresh the snapshot once the editor starts.

## Manual terminal verification

Use disposable files, not personal or credential data:

```bash
repo=$(pwd)
tmpdir=$(mktemp -d)
printf 'one\ntwo\nthree\n' > "$tmpdir/one.txt"
printf 'alpha\nbeta\ngamma\n' > "$tmpdir/two.txt"
(cd "$tmpdir" && "$repo/scripts/run.sh" one.txt)
rm -rf "$tmpdir"
```

Verify:

- The editor opens only in an interactive terminal; piping or CI gets the summary behavior.
- Arrow keys move across and between lines.
- PageUp and PageDown move by one visible editor page.
- Ctrl-PageUp and Ctrl-PageDown select the previous/next editor tab. With only one tab open they report that only
  one tab is open.
- Ctrl-F4 closes the active editor tab. Closing the only tab is refused; closing a dirty tab requires pressing
  Ctrl-F4 a second time.
- When more than one tab is open, a compact tab strip appears below the top header. The active tab is highlighted,
  and dirty tabs show `*`.
- With the sidebar open, select `two.txt` and press Enter. The file opens as a second active tab without replacing
  `one.txt`; Ctrl-PageUp/Ctrl-PageDown switches between the two tabs.
- Click visible tab labels in the tab strip and verify the active tab changes.
- Edit one tab and verify the tab strip marks it dirty. Pressing Ctrl-F4 once warns, pressing Ctrl-F4 again closes
  only that tab, and the other tab remains open.
- F10 -> Tabs exposes previous-tab, next-tab, close-tab, and an Open sidebar file as tab reminder.
- F10 -> Workspace can save the current tab set, save a named project, manage saved projects in a visible panel,
  open or delete a named project through typed prompts, open the current workspace snapshot, and toggle
  restore-last-workspace. In save/open/delete prompts, Up/Down cycles existing project names.
- Save current from the Workspace menu, quit, relaunch `kfnotepad` with restore-last-workspace enabled, and verify
  the current workspace files reopen. Disable restore-last-workspace when done if you do not want argument-free
  launch to restore files.
- F10 opens the keyboard menu; Left/Right, Tab/Shift-Tab, Up/Down, Home/End, Enter, and Esc operate the menu without
  mouse capture, and dropdown items show shortcut hints where direct keybindings exist.
- F2 opens the command palette. Type `wrap`, `reader`, `ctrl-s`, or another command/group/shortcut fragment; verify
  the results filter and Enter runs the selected command.
- File -> New file and Ctrl-N outside the sidebar create a new clean untitled tab without writing a file until save.
- The bottom footer remains inside one terminal row and shows a compact hint. The Help menu opens a maintained
  built-in help document and keeps a compact shortcut reference.
- File menu Save writes through the same save path as Ctrl-S; File menu Quit uses the same dirty confirmation as
  Ctrl-Q.
- The Edit menu exposes delete previous word and delete next word.
- The Go menu exposes previous word and next word.
- Home and End move to the start/end of the current line.
- Ctrl-A and Ctrl-E move to the start/end of the current line.
- Ctrl-Left and Ctrl-Right move by word.
- Ctrl-Home and Ctrl-End move to the start/end of the document.
- Typing inserts text at the cursor.
- Horizontal cursor movement, Backspace, Delete, and overwrite-mode replacement treat combining marks, emoji ZWJ
  sequences, and regional-indicator flag pairs as single grapheme clusters.
- Enter splits a line.
- Backspace and Delete remove text and join lines at boundaries.
- Ctrl-Backspace and Ctrl-Delete delete by word.
- Ctrl-K deletes to the end of the current line without joining the next line.
- In WezTerm, Ghostty, or another modern terminal, Ctrl-Backspace is delivered as a distinct modified Backspace
  event or the terminal-specific limitation is documented.
- Ctrl-Z undoes edits made since the last save.
- Ctrl-Y redoes the last undone edit, unless a new edit cleared redo history.
- Ctrl-F prompts for a search query and jumps to the next match when Enter is pressed. Search is case-insensitive by
  default, Ctrl-Shift-F toggles exact-case mode, and Up/Down recall session search history while the prompt is active.
- F3 repeats the last accepted non-empty search forward using the current case mode.
- Shift-F3 repeats the last accepted non-empty search backward using the current case mode.
- Ctrl-G prompts for a line number and jumps to that one-based line when Enter is pressed.
- While entering a search query or line number, the visible cursor appears in the status-line prompt.
- Mouse left-click moves the cursor in the visible body, including wrapped visual rows, and operates visible
  top-menu/dropdown items.
- Mouse wheel over the visible body moves the cursor three rows and scrolls the viewport through existing
  cursor-following behavior.
- Mouse movement and unsupported mouse events are ignored without forcing a redraw.
- The app does not provide an in-app selection, clipboard, or copy command yet.
- Ctrl-L hides and restores the line-number gutter without modifying the edited file and persists the preference.
- Ctrl-T cycles the runtime chrome theme through `nocturne`, `aurora`, `pastel`, `terminal`, `abyss`, and `terror`
  without modifying the edited file and persists the preference.
- Ctrl-Shift-T cycles the syntax highlighting theme through the same preset names without modifying the edited file
  and persists the preference.
- Ctrl-W toggles word wrap without modifying the edited file and persists the preference.
- Ctrl-B opens a dismissable file sidebar rooted at the process current working directory the first time it opens,
  then reopens at the last visited sidebar directory for the current editor session.
- While open, the sidebar reserves left-side columns and the editor renders in the remaining pane.
- The sidebar lists `..`, subdirectories, and regular files; `..` and subdirectories navigate the sidebar, and
  regular files open through the same UTF-8/symlink/size validation as direct `kfnotepad FILE` opens.
- While the sidebar is open, Enter on a regular file opens or focuses a tab without replacing a dirty current
  document. Ctrl-Enter keeps the same behavior when the terminal delivers it, but is not required.
- While the sidebar is open, Ctrl-N creates a new file in the selected directory, or in the current sidebar directory
  when a file or `..` is selected. Ctrl-D creates a new directory using the same target rule. Creating inside a
  selected directory moves the sidebar view into that directory so the new child is visible. Delete starts a typed
  confirmation; confirmed file/directory deletion moves the selected path to the operating system Trash/Recycle Bin,
  with directories moving recursively only after confirming with `yes`.
- Mouse wheel over the sidebar moves the selected entry up/down without wrapping at the first or last entry.
- Selecting a file while the current buffer is dirty opens the selected file as another tab; the dirty tab remains
  open and marked dirty.
- Ctrl-S saves, exits only when Ctrl-Q is pressed, and the file contains the expected text afterward.
- With unsaved changes, one Ctrl-Q shows a warning and the second Ctrl-Q exits.
- Scrolling keeps the cursor visible in a file longer than the terminal viewport.
- Tabs, combining marks, CJK characters, and emoji remain visually aligned with cursor placement.
- The status line reports the current line and column.
- The active text cell is visibly highlighted in addition to the terminal cursor.
- The header keeps saved/modified visible even when the file path is long.
- In horizontal-scroll mode, the status line reports `x:0` at the left edge and a one-based `x:N` offset after
  scrolling right; in wrap mode it reports `x:0`.
- On exit, the terminal prompt and cursor are restored.

Managed notes smoke with disposable data:

```bash
tmpdata=$(mktemp -d)
XDG_DATA_HOME="$tmpdata" ./scripts/run.sh --note "Daily Note"
XDG_DATA_HOME="$tmpdata" ./scripts/run.sh --notes
rm -rf "$tmpdata"
```

Verify:

- `--note "Daily Note"` opens `daily-note.md` under the disposable data directory.
- Typing text, Ctrl-S, and Ctrl-Q persist the note.
- `--notes` lists `daily-note.md`.
- The disposable data directory can be removed after verification.
