impl KfnotepadGui {
    pub(super) fn refresh_all_file_snapshots(&mut self) {
        self.file_snapshots.clear();
        for tile in &self.workspace.tiles {
            if let Ok(Some(snapshot)) = gui_file_snapshot(&tile.document.path) {
                self.file_snapshots.insert(tile.id, snapshot);
            }
        }
    }

    pub(super) fn refresh_file_snapshot_for_tile(&mut self, tile_id: GuiTileId) {
        let Some(path) = self
            .workspace
            .tile(tile_id)
            .map(|tile| tile.document.path.clone())
        else {
            self.file_snapshots.remove(&tile_id);
            return;
        };

        match gui_file_snapshot(&path) {
            Ok(Some(snapshot)) => {
                self.file_snapshots.insert(tile_id, snapshot);
            }
            Ok(None) | Err(_) => {
                self.file_snapshots.remove(&tile_id);
            }
        }
    }

    pub(super) fn external_file_check_candidates(&self) -> Vec<GuiExternalFileCheckCandidate> {
        self.workspace
            .tiles
            .iter()
            .map(|tile| GuiExternalFileCheckCandidate {
                tile_id: tile.id,
                path: tile.document.path.clone(),
                dirty: tile.document.buffer.is_dirty(),
                previous_snapshot: self.file_snapshots.get(&tile.id).cloned(),
            })
            .collect()
    }

    pub(super) fn request_external_file_check(&self) -> Task<Message> {
        let candidates = self.external_file_check_candidates();
        Task::perform(
            async move { check_external_file_changes_async(candidates).await },
            Message::ExternalFileCheckCompleted,
        )
    }

    pub(super) fn apply_external_file_check_result(&mut self, result: GuiExternalFileCheckResult) {
        match result {
            GuiExternalFileCheckResult::SnapshotInitialized { tile_id, snapshot } => {
                self.file_snapshots.insert(tile_id, snapshot);
            }
            GuiExternalFileCheckResult::DirtyChanged {
                tile_id,
                path,
                snapshot,
            } => {
                self.file_snapshots.insert(tile_id, snapshot);
                self.external_edit_locks.insert(tile_id);
                self.status_message = format!(
                    "external change detected for {}; save or close local edits before refresh",
                    gui_file_name_label(&path)
                );
            }
            GuiExternalFileCheckResult::Reloaded {
                tile_id,
                path,
                snapshot,
                document,
            } => {
                self.replace_tile_document_from_external_change(tile_id, document);
                self.file_snapshots.insert(tile_id, snapshot);
                self.external_edit_locks.insert(tile_id);
                self.status_message =
                    format!("external update loaded: {}", gui_file_name_label(&path));
            }
            GuiExternalFileCheckResult::LoadFailed {
                tile_id,
                path,
                message,
            } => {
                if self.workspace.tile(tile_id).is_some() {
                    self.status_message =
                        format!("external update skipped for {}: {message}", path.display());
                }
            }
        }
    }

    #[cfg(test)]
    pub(super) fn poll_external_file_changes(&mut self) {
        let candidates = self.external_file_check_candidates();
        let results = check_external_file_changes(candidates);
        for result in results {
            self.apply_external_file_check_result(result);
        }
    }
}
