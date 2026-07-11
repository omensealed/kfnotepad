pub fn tui_help_document_text() -> &'static str {
    r#"# kfnotepad help

kfnotepad is a local UTF-8 text-file editor for modern terminals. It edits normal files on disk; there is no database, account, sync service, or autosave.

## Editor basics

- Type normally in the active document.
- Ctrl-S saves the active document.
- Ctrl-N creates a new untitled file tab without writing it until save.
- Ctrl-Q quits; modified documents require Ctrl-Q twice.
- F2 opens the command palette. Type part of a command, shortcut, or menu group; Up/Down chooses a result; Enter runs it; Esc closes it.
- F10 opens the menu bar. Left/Right or Tab/Shift-Tab changes groups, Up/Down chooses an item, Home/End jumps within a menu, and Enter runs the selected item.
- Mouse clicks move the cursor and operate menu items.
- Clipboard copy and paste use the terminal's native selection and paste behavior.

## Movement and editing

- Arrow keys move the cursor.
- Home and End move within the current line.
- Ctrl-A and Ctrl-E also move to the start or end of the current line.
- Ctrl-Home and Ctrl-End move to the document start or end.
- Ctrl-Left and Ctrl-Right move by word.
- PageUp and PageDown move by one visible page.
- Enter splits the current line.
- Backspace deletes before the cursor.
- Delete removes at the cursor.
- Ctrl-Backspace and Ctrl-Delete delete by word.
- Ctrl-K deletes to the end of the current line.
- Tab inserts one four-space indentation level.
- Shift-Tab removes up to one indentation level before the cursor.
- Insert toggles overwrite mode. In overwrite mode, printable characters replace the character under the cursor when possible and insert normally at line end.
- Ctrl-Z undoes edits since the last save.
- Ctrl-Y redoes the last undone edit.

## Search and navigation

- Ctrl-F starts a search prompt.
- Search is case-insensitive by default.
- Ctrl-Shift-F toggles exact-case search.
- Accepted matches are highlighted.
- Up and Down recall the last ten accepted search queries while the search prompt is active.
- F3 repeats the last search forward.
- Shift-F3 repeats the last search backward.
- Ctrl-G opens the go-to-line prompt.
- Go menu entries can jump by page, document edge, or word.

## Tabs

- File > New file or Ctrl-N creates a new untitled file tab. The file is not written until Save.
- Ctrl-PageUp and Ctrl-PageDown switch documents.
- Ctrl-F4 closes the active tab.
- Closing a modified tab requires confirmation.
- Ctrl-B opens the file sidebar; Enter opens or focuses the selected sidebar file as a tab.
- The tab strip appears when more than one tab is open, and visible tab labels can be clicked with the mouse.

## File sidebar

- Ctrl-B toggles the file sidebar. Reopening it returns to the last visited sidebar directory in this session.
- Up and Down move the sidebar selection.
- Enter opens or focuses a selected file as a tab, or navigates into a directory.
- Ctrl-Enter also opens or focuses the selected sidebar file as a tab when the terminal delivers it.
- Ctrl-N creates a child file in the selected directory or current sidebar directory.
- Ctrl-D creates a child directory in the selected directory or current sidebar directory.
- Delete starts a typed yes confirmation and moves files/directories to the operating system Trash/Recycle Bin.
- Directory deletion warns because nested files and directories are moved too.
- Symlinks are not opened or deleted by the default file actions.
- Escape cancels sidebar prompts or closes the sidebar.

## Workspaces

- F10 -> Workspace saves, lists, opens, deletes, and restores workspace projects.
- Workspace projects store normal file paths and active-tab selection.
- Save named, Open project, and Delete project prompts support Up and Down to cycle saved project names.
- Delete project requires typing `yes` before the project snapshot is removed.
- Opening a project into a dirty session requires typed confirmation.
- Restore last workspace uses the existing persisted preference and opens the saved TUI current workspace on argument-free interactive launch.
- TUI workspace projects live under the `workspaces/tui` config subdirectory, separate from GUI tile workspaces, while using the same path-only project format.
- TUI workspace restore ignores GUI tile geometry and opens the project files as terminal tabs.

## Reader mode

- Ctrl-R toggles reader mode.
- View -> Reader mode also toggles it.
- View -> Reader slower and View -> Reader faster adjust the persisted lines-per-minute speed.
- Reader mode scrolls the active document downward without moving the edit cursor.
- Reader mode stops at the end of the document, when you edit, when you switch tabs, when you open a file, or when you open a workspace project.

## Themes and syntax

- Ctrl-T cycles the terminal chrome theme.
- Ctrl-Shift-T cycles the syntax highlighting theme independently.
- Ctrl-L toggles line numbers.
- Ctrl-W toggles word wrap.
- Syntax highlighting is extension-based with a plain-text fallback.

## Managed notes

- `kfnotepad --note TITLE` creates or opens a managed Markdown note under the local XDG data directory.
- `kfnotepad --notes` lists managed note filenames.
- Managed notes are normal Markdown files. They are not stored in a database.

## Safety and limits

- kfnotepad opens local UTF-8 text files only.
- Directories, symlinks, missing files, non-regular files, non-UTF-8 files, and files above the configured size limit are rejected by the safe open path.
- Save uses the same local atomic file adapter as the GUI. Existing save targets must be regular files.
- If a file changed on disk since open or last save, kfnotepad refuses to overwrite it silently.
- Saved text is normalized to LF line endings.
- The editor does not use network services, user accounts, or background sync.

## Troubleshooting

- If a desktop, terminal, multiplexer, or shell intercepts a shortcut before kfnotepad receives it, use F2 and run the command by name.
- If Ctrl-Left, Ctrl-Right, Ctrl-Backspace, or Ctrl-Delete do not work, confirm the terminal supports modified key reporting.
- If colors or icons look wrong, check the selected theme and terminal font.
- If workspace restore opens unexpected files, disable Restore last workspace from F10 -> Workspace, then save a fresh current workspace after opening the intended tabs.
"#
}
