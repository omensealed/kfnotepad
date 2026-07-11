impl KfnotepadGui {
    pub(super) fn request_save_active_tile_async(&mut self) -> Task<Message> {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save failed: no active pane".to_string();
            return Task::none();
        };

        let Some(document) = self
            .workspace
            .tile(tile_id)
            .map(|tile| tile.document.clone())
        else {
            self.status_message = "save failed: no active tile".to_string();
            return Task::none();
        };

        let expected_text = document.buffer.to_text();
        self.status_message = format!("saving {}", gui_file_name_label(&document.path));

        Task::perform(
            async move {
                let mut document = document;
                save_text_document(&mut document).map_err(|error| error.to_string())
            },
            move |result| Message::SaveActiveTileCompleted {
                tile_id,
                expected_text,
                result,
            },
        )
    }

    pub(super) fn request_save_active_tile_as(&mut self, path: PathBuf) -> Task<Message> {
        self.sync_active_editor_to_document();
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save as failed: no active pane".to_string();
            return Task::none();
        };

        if let Some(open_tile_id) = self.open_tile_id_for_path(&path) {
            if open_tile_id != tile_id {
                self.focus_or_restore_existing_tile(open_tile_id, &path);
                self.status_message = format!(
                    "save as refused: {} is already open in another tile",
                    path.display()
                );
                return Task::none();
            }
        }

        let Some(document) = self
            .workspace
            .tile(tile_id)
            .map(|tile| tile.document.clone())
        else {
            self.status_message = "save as failed: no active tile".to_string();
            return Task::none();
        };

        let original_path = document.path.clone();
        let mut document = document;
        let clear_snapshot = !gui_paths_refer_to_same_file(&original_path, &path);
        let expected_text = document.buffer.to_text();

        document.path = path.clone();
        if clear_snapshot {
            document.buffer.set_file_snapshot(None);
        }

        self.status_message = format!("saving as {}", gui_file_name_label(&path));

        Task::perform(
            async move {
                let mut document = document;
                save_text_document(&mut document).map_err(|error| error.to_string())
            },
            move |result| Message::SaveActiveTileAsCompleted {
                tile_id,
                original_path,
                requested_path: path,
                expected_text,
                clear_snapshot,
                result,
            },
        )
    }
}
