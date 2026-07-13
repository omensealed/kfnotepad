//! Built-in local help document content.

pub(crate) const GUI_HELP_DOCUMENT_TEXT: &str = r#"# kfnotepad help

kfnotepad is a local UTF-8 text-file editor. The terminal editor and Iced GUI both edit normal files on disk; there is no database, account, sync service, or autosave.

## Files

- Use the Files panel to browse from the current working directory.
- Single-click a file or directory to select it.
- Double-click a file to open it in a tile.
- Double-click a directory to make it the active tree location.
- Use the parent and refresh buttons to move up or reload the tree.
- Create file and create folder use the selected directory when a directory is selected; otherwise they use the current Files root.
- Opening a file that is already open focuses or restores the existing tile instead of opening a duplicate.

## Tiles

- File > New tile or Ctrl-N creates an untitled document tile.
- File > Open or Ctrl-O opens local files.
- File > Save or Ctrl-S saves the active tile.
- File > Close tile or Ctrl-F4 closes the active tile. Unsaved tiles ask for confirmation.
- Tile > Equalize tiles arranges open tiles into an even grid.
- Tile controls can move, minimize, maximize, restore, or close a tile. Hover a tile titlebar to show those controls.
- Minimized tiles move to the tray below the menu and can be restored from there.
- Ctrl-Shift-Arrow moves the active tile. Ctrl-M minimizes/restores it. Ctrl-Shift-M maximizes/restores it.

## Editing

- Type normally in the active tile.
- Ctrl-Z and Ctrl-Y undo and redo.
- Ctrl-C, Ctrl-X, Ctrl-V copy, cut, and paste selected text.
- Ctrl-A selects all text.
- Insert toggles overwrite mode.
- Home and End move within a line; Ctrl-Home and Ctrl-End move to the document edges.
- Ctrl-Left and Ctrl-Right move by word.
- Ctrl-Backspace and Ctrl-Delete delete by word.
- Ctrl-K deletes to the end of the current line.
- PageUp and PageDown move the active document viewport.
- Mouse click moves the cursor; drag selects text; mouse wheel scrolls the hovered tile.
- IME committed text is supported, and preedit text is shown transiently at the cursor.

## Search and navigation

- Ctrl-F focuses Find.
- Search is case-insensitive by default.
- Use the exact-case toggle in the Find row or Preferences to make search case-sensitive.
- F3 finds the next match; Shift-F3 finds the previous match.
- Recent Find queries are kept in a session-only history dropdown when the Find field is empty.
- Ctrl-G focuses Go to line.
- The Nav menu can jump to the top or bottom of the active document.
- The active search match is selected and highlighted in the editor.

## Reader mode

- Ctrl-R, View > Reader mode, the header reader button, or Preferences toggles reader mode.
- Reader mode auto-scrolls the active document down without editing or saving it.
- Reader speed is configured in Preferences as lines per minute.
- Reader mode stops at the end of the document.

## Themes and syntax

- Ctrl-T cycles the app theme.
- Ctrl-Shift-T cycles the syntax highlighting theme independently from the app theme.
- Preferences can also cycle syntax theme and configure line numbers, wrapping, editor font, editor size, UI size, exact-case search, reader mode, reader speed, and restore-last-workspace.
- Built-in app and syntax theme names are Nocturne, Aurora, Pastel, Terminal, Abyss, and Terror.
- Syntax colors are adjusted for readable contrast against the selected app theme.

## Workspaces and preferences

- The Workspaces panel can save and reopen a group of open tiles and layout.
- Save current updates the deterministic current workspace used by restore-last-workspace.
- Save named stores a project under the entered name.
- Saved projects can open in the current window or in a new kfnotepad GUI process.
- Deleting a saved project requires confirmation.
- Restore last workspace is opt-in from Preferences. When enabled, argument-free startup reopens the latest saved current workspace when it is still valid.

## Managed notes

- Notes > Open note creates or opens a managed Markdown note.
- Notes > List notes shows existing managed notes.
- Notes can be deleted from the list after confirmation.
- Managed notes are normal `.md` files under the local XDG data directory.

## External changes

If an already-open clean file changes on disk, kfnotepad refreshes the tile and locks editing to avoid overwriting the outside change. Locked tiles still allow scrolling and further external refreshes. Use the unlock button in the tile titlebar when you are ready to edit locally again. Dirty local buffers are not overwritten by external refresh.

## Saving and safety

- Save uses the same atomic local-file adapter as the terminal editor.
- Open rejects missing files, directories, symlinks, non-UTF-8 files, and files larger than 8 MiB.
- Save rejects symlink targets and preserves existing file permissions where possible.
- Unsaved buffers remain in memory until saved.
"#;
