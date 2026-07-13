impl KfnotepadGui {
    pub(in crate::gui::app::state) fn open_managed_note_by_title(&mut self, title: &str) -> bool {
        let Some(notes_dir) = self.notes_dir.clone() else {
            self.status_message =
                "managed notes unavailable: cannot resolve data directory".to_string();
            return false;
        };

        match open_or_create_managed_note(&notes_dir, title) {
            Ok(document) => {
                let path = document.path.clone();
                self.open_document_in_new_pane(document, format!("opened note {}", path.display()))
            }
            Err(error) => {
                self.status_message = format!("managed note failed: {error}");
                false
            }
        }
    }

    pub(in crate::gui::app::state) fn list_managed_notes_panel(&mut self) {
        let Some(notes_dir) = self.notes_dir.as_deref() else {
            self.notes_panel = None;
            self.pending_managed_note_delete = None;
            self.status_message =
                "managed notes unavailable: cannot resolve data directory".to_string();
            return;
        };

        match list_managed_notes(notes_dir) {
            Ok(notes) => {
                let count = notes.len();
                self.notes_panel = Some(notes);
                self.pending_managed_note_delete = None;
                self.status_message = format!("managed notes: {count}");
            }
            Err(error) => {
                self.notes_panel = None;
                self.pending_managed_note_delete = None;
                self.status_message = format!("managed notes failed: {error}");
            }
        }
    }

    pub(in crate::gui::app::state) fn open_managed_note_from_panel(&mut self, index: usize) {
        let Some(note) = self
            .notes_panel
            .as_ref()
            .and_then(|notes| notes.get(index))
            .cloned()
        else {
            return;
        };

        if self.open_path_in_new_pane(note.path) {
            self.notes_panel = None;
            self.pending_managed_note_delete = None;
        }
    }

    pub(in crate::gui::app::state) fn delete_managed_note_from_panel(&mut self, index: usize) {
        let Some(note) = self
            .notes_panel
            .as_ref()
            .and_then(|notes| notes.get(index))
            .cloned()
        else {
            return;
        };

        if self
            .workspace
            .tiles
            .iter()
            .any(|tile| tile.document.path == note.path)
        {
            self.pending_managed_note_delete = None;
            self.status_message = format!("close note tile before deleting {}", note.file_name);
            return;
        }

        if self.pending_managed_note_delete.as_deref() != Some(note.path.as_path()) {
            self.pending_managed_note_delete = Some(note.path.clone());
            self.pending_project_open = None;
            self.pending_project_delete = None;
            self.pending_close_tile = None;
            self.pending_app_quit = false;
            self.status_message = format!("delete note {}? click delete again", note.file_name);
            return;
        }

        self.pending_managed_note_delete = None;
        let Some(notes_dir) = self.notes_dir.clone() else {
            self.status_message =
                "managed note delete failed: cannot resolve data directory".to_string();
            return;
        };

        match delete_managed_note(&notes_dir, &note.path) {
            Ok(ManagedNoteDeleteResult::Deleted) => {
                self.list_managed_notes_panel();
                self.status_message = format!("managed note moved to trash: {}", note.file_name);
            }
            Ok(ManagedNoteDeleteResult::Missing) => {
                self.list_managed_notes_panel();
                self.status_message = format!("managed note already missing: {}", note.file_name);
            }
            Err(error) => {
                self.status_message = format!("managed note delete failed: {error}");
            }
        }
    }
}
