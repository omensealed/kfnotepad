impl KfnotepadGui {
pub(super) fn delete_selected_browser_entry(&mut self) -> Task<Message> {
        if !self.browser_visible || self.left_panel.mode != GuiLeftPanelMode::Files {
            return Task::none();
        }
        let Some(entry) = self.selected_browser_action_entry() else {
            self.status_message = "delete failed: no file-browser selection".to_string();
            return Task::none();
        };
        if entry.kind == FileSidebarEntryKind::Parent {
            self.pending_browser_delete = None;
            self.status_message = "delete failed: cannot delete parent shortcut".to_string();
            return Task::none();
        }
        if self.path_is_open_in_workspace(&entry.path) {
            self.pending_browser_delete = None;
            self.status_message =
                format!("close open tile before deleting {}", entry.path.display());
            return Task::none();
        }
        if entry.kind == FileSidebarEntryKind::Directory
            && self.directory_contains_open_tile(&entry.path)
        {
            self.pending_browser_delete = None;
            self.status_message = format!(
                "close open tiles inside {} before deleting directory",
                entry.path.display()
            );
            return Task::none();
        }

        if self.pending_browser_delete.as_deref() != Some(entry.path.as_path()) {
            self.pending_browser_delete = Some(entry.path.clone());
            self.pending_project_open = None;
            self.pending_project_delete = None;
            self.pending_managed_note_delete = None;
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = if entry.kind == FileSidebarEntryKind::Directory {
                format!(
                    "delete directory {} and all subdirectories/files? click delete again",
                    entry.path.display()
                )
            } else {
                format!("delete file {}? click delete again", entry.path.display())
            };
            return Task::none();
        }

        self.pending_browser_delete = None;
        match delete_browser_path(&entry.path, entry.kind) {
            Ok(()) => {
                let refresh_task = self.refresh_file_browser();
                self.browser_selected_path = None;
                self.status_message = match entry.kind {
                    FileSidebarEntryKind::Directory => {
                        format!("moved directory to trash {}", entry.path.display())
                    }
                    FileSidebarEntryKind::File => {
                        format!("moved file to trash {}", entry.path.display())
                    }
                    FileSidebarEntryKind::Parent => unreachable!("parent handled above"),
                };
                refresh_task
            }
            Err(error) => {
                self.status_message = format!("delete failed: {error}");
                Task::none()
    }
}
}
    }
