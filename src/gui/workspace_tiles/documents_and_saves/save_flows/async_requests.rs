impl KfnotepadGui {
    pub(super) fn request_save_active_tile_async(&mut self) -> Task<Message> {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save failed: no active pane".to_string();
            return Task::none();
        };
        if self.save_in_flight.contains(&tile_id) {
            self.save_requested_after_in_flight.insert(tile_id);
            self.status_message = "save queued after current save".to_string();
            return Task::none();
        }
        self.request_save_tile_async(tile_id)
    }

    pub(super) fn request_save_tile_async(&mut self, tile_id: GuiTileId) -> Task<Message> {
        let Some(pane) = self
            .panes
            .iter()
            .find_map(|(pane, state)| (state.tile_id == tile_id).then_some(*pane))
        else {
            self.status_message = "save failed: tile pane no longer exists".to_string();
            return Task::none();
        };
        let Some((synced_tile_id, text)) = self.sync_pane_to_document_text(pane) else {
            self.status_message = "save failed: tile pane no longer exists".to_string();
            return Task::none();
        };
        debug_assert_eq!(synced_tile_id, tile_id);

        let Some((path, source_revision, expected_snapshot)) = self
            .workspace
            .tile(tile_id)
            .map(|tile| {
                (
                    tile.document.path.clone(),
                    tile.document.buffer.edit_revision(),
                    tile.document.buffer.file_snapshot().cloned(),
                )
            })
        else {
            self.status_message = "save failed: no active tile".to_string();
            return Task::none();
        };

        self.status_message = format!("saving {}", gui_file_name_label(&path));
        self.save_in_flight.insert(tile_id);

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    save_text_snapshot(&path, &text, expected_snapshot.as_ref()).map(|snapshot| {
                        GuiSaveResult {
                            source_revision,
                            snapshot,
                        }
                    })
                })
                .await
                .map_err(|error| format!("save worker failed: {error}"))?
                .map_err(|error| error.to_string())
            },
            move |result| Message::SaveActiveTileCompleted { tile_id, result },
        )
    }

    #[cfg(not(test))]
    pub(super) fn request_save_active_tile_as(&mut self, path: PathBuf) -> Task<Message> {
        let Some(tile_id) = self
            .panes
            .get(self.active_pane)
            .map(|pane_state| pane_state.tile_id)
        else {
            self.status_message = "save as failed: no active pane".to_string();
            return Task::none();
        };
        if self.save_in_flight.contains(&tile_id) {
            self.status_message = "save as refused: save already in progress".to_string();
            return Task::none();
        }
        let Some((_, text)) = self.sync_pane_to_document_text(self.active_pane) else {
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

        let Some((original_path, source_revision, current_snapshot)) = self
            .workspace
            .tile(tile_id)
            .map(|tile| {
                (
                    tile.document.path.clone(),
                    tile.document.buffer.edit_revision(),
                    tile.document.buffer.file_snapshot().cloned(),
                )
            })
        else {
            self.status_message = "save as failed: no active tile".to_string();
            return Task::none();
        };

        let clear_snapshot = !gui_paths_refer_to_same_file(&original_path, &path);
        let expected_snapshot = (!clear_snapshot).then_some(current_snapshot).flatten();

        self.status_message = format!("saving as {}", gui_file_name_label(&path));
        self.save_in_flight.insert(tile_id);
        let save_path = path.clone();

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    save_text_snapshot(&save_path, &text, expected_snapshot.as_ref()).map(
                        |snapshot| GuiSaveResult {
                            source_revision,
                            snapshot,
                        },
                    )
                })
                .await
                .map_err(|error| format!("save worker failed: {error}"))?
                .map_err(|error| error.to_string())
            },
            move |result| Message::SaveActiveTileAsCompleted {
                tile_id,
                requested_path: path,
                result,
            },
        )
    }
}
