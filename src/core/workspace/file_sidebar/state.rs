//! Sidebar loading, selection, scrolling, and mouse-row mapping.

use super::*;

impl FileSidebarState {
    pub fn load(current_dir: PathBuf) -> Result<Self, FileSidebarError> {
        let current_dir = current_dir.canonicalize().unwrap_or(current_dir);
        Ok(Self {
            entries: list_file_sidebar_entries(&current_dir)?,
            current_dir,
            selected: 0,
            scroll: 0,
        })
    }

    pub fn selected_entry(&self) -> Option<&FileSidebarEntry> {
        self.entries.get(self.selected)
    }

    pub fn selected_entry_for_mouse_row(&mut self, row: u16) -> Option<FileSidebarEntry> {
        if row == 0 {
            return None;
        }
        let index = self.scroll + row as usize - 1;
        let entry = self.entries.get(index)?.clone();
        self.selected = index;
        Some(entry)
    }

    pub fn select_previous_wrapping(&mut self, visible_rows: usize) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = if self.selected == 0 {
            self.entries.len() - 1
        } else {
            self.selected - 1
        };
        self.keep_selection_visible(visible_rows);
    }

    pub fn select_next_wrapping(&mut self, visible_rows: usize) {
        if self.entries.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.entries.len();
        self.keep_selection_visible(visible_rows);
    }

    pub fn scroll_selection_up(&mut self, visible_rows: usize) -> bool {
        if self.entries.is_empty() || self.selected == 0 {
            return false;
        }
        self.selected -= 1;
        self.keep_selection_visible(visible_rows);
        true
    }

    pub fn scroll_selection_down(&mut self, visible_rows: usize) -> bool {
        if self.entries.is_empty() || self.selected + 1 >= self.entries.len() {
            return false;
        }
        self.selected += 1;
        self.keep_selection_visible(visible_rows);
        true
    }

    pub fn keep_selection_visible(&mut self, visible_rows: usize) {
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        let visible = visible_rows.max(1);
        if self.selected >= self.scroll + visible {
            self.scroll = self.selected.saturating_sub(visible - 1);
        }
    }
}
