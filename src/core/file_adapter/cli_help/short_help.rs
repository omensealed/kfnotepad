//! Concise command-line usage and terminal shortcut help.

pub fn help_text() -> &'static str {
    r#"kfnotepad 0.1.1

Usage:
  kfnotepad [FILE]
  kfnotepad --note TITLE
  kfnotepad --notes
  kfnotepad --help
  kfnotepad --version

Behavior:
  With FILE in an interactive terminal, opens the editor.
  With FILE in a non-interactive context, prints a read-only summary.
  With --note TITLE, creates or opens a managed Markdown note under the local data directory.
  With --notes, lists managed note filenames in deterministic order.
  With no FILE, verifies the executable can launch unless workspace restore is enabled.

Editor keys:
  Arrow keys move the cursor.
  Mouse clicks move the cursor, select visible tabs, and operate menu items.
  Ctrl-B toggles the file sidebar.
  Ctrl-N creates a new untitled file tab without writing it until save.
  In the file sidebar, Enter opens or focuses the selected file as a tab.
  In the file sidebar, Ctrl-N creates a child file, Ctrl-D creates a directory, and Delete prompts for removal.
  Ctrl-PageUp and Ctrl-PageDown switch tabs; Ctrl-F4 closes the active tab.
  F10 -> Workspace saves, lists, opens, deletes, and restores TUI workspace projects.
  Ctrl-Left and Ctrl-Right move by word.
  PageUp and PageDown move by one visible page.
  F2 opens the command palette for typed access to menu commands.
  F10 opens the keyboard menu.
  Home, End, Ctrl-A, and Ctrl-E move within the current line.
  Ctrl-Home and Ctrl-End move to the start or end of the document.
  Printable characters insert text.
  Tab inserts one indentation level.
  Shift-Tab removes one indentation level before the cursor.
  Enter splits the current line.
  Backspace deletes before the cursor.
  Delete removes at the cursor.
  Ctrl-Backspace and Ctrl-Delete delete by word.
  Ctrl-K deletes to the end of the current line.
  Ctrl-Z undoes recent edits since the last save; undo history is bounded by count and memory.
  Ctrl-Y redoes the last undone edit.
  Ctrl-F searches text; accepted matches are highlighted.
  Search defaults to ignore case; Ctrl-Shift-F toggles exact-case search.
  Up and Down recall the last ten search queries while the search prompt is active.
  F3 repeats the last search forward.
  Shift-F3 repeats the last search backward.
  Ctrl-G goes to a line number.
  Ctrl-L toggles line numbers.
  Ctrl-T cycles built-in themes.
  Ctrl-Shift-T cycles the syntax highlighting theme independently.
  Ctrl-R toggles reader mode auto-scroll.
  Ctrl-W toggles word wrap.
  Ctrl-S saves.
  Ctrl-Q quits; dirty buffers require Ctrl-Q twice.
"#
}
