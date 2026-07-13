impl KfnotepadGui {
    pub(in crate::gui::app::state) fn refresh_all_file_snapshots(&mut self) {
        self.file_snapshots.clear();
        for tile in &self.workspace.tiles {
            if let Ok(Some(snapshot)) = gui_file_snapshot(&tile.document.path) {
                self.file_snapshots.insert(tile.id, snapshot);
            }
        }
    }

    pub(in crate::gui::app::state) fn refresh_file_snapshot_for_tile(&mut self, tile_id: GuiTileId) {
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

    pub(in crate::gui::app::state) fn external_file_check_candidates(
        &self,
        force_deep_check: bool,
    ) -> Vec<GuiExternalFileCheckCandidate> {
        self.workspace
            .tiles
            .iter()
            .map(|tile| GuiExternalFileCheckCandidate {
                tile_id: tile.id,
                path: tile.document.path.clone(),
                dirty: tile.document.buffer.is_dirty(),
                previous_snapshot: self.file_snapshots.get(&tile.id).cloned(),
                force_deep_check,
            })
            .collect()
    }

    pub(in crate::gui::app::state) fn request_external_file_check(&mut self) -> Task<Message> {
        if self.external_file_check_in_flight {
            return Task::none();
        }

        self.external_file_check_tick = self.external_file_check_tick.wrapping_add(1);
        self.sync_external_file_watcher();

        let fallback_tick = self.external_file_check_tick.is_multiple_of(60);
        let watcher_paths = self.drain_external_file_watcher();
        let watcher_active = self.external_file_watcher.is_some();
        if watcher_active && watcher_paths.is_empty() && !fallback_tick {
            return Task::none();
        }

        let force_deep_check = !watcher_paths.is_empty() || (!watcher_active && fallback_tick);
        let mut candidates = self.external_file_check_candidates(force_deep_check);
        if !watcher_paths.is_empty() {
            candidates.retain(|candidate| {
                watcher_paths
                    .iter()
                    .any(|event_path| watcher_event_matches_path(event_path, &candidate.path))
            });
        }
        if candidates.is_empty() {
            return Task::none();
        }

        self.external_file_check_in_flight = true;
        Task::perform(
            async move { check_external_file_changes_async(candidates).await },
            Message::ExternalFileCheckCompleted,
        )
    }

    pub(in crate::gui::app::state) fn sync_external_file_watcher(&mut self) {
        let paths = self
            .workspace
            .tiles
            .iter()
            .map(|tile| tile.document.path.clone())
            .collect::<Vec<_>>();
        let result = self
            .external_file_watcher
            .as_mut()
            .map(|watcher| watcher.sync_paths(&paths));
        if let Some(Err(error)) = result {
            self.external_file_watcher = None;
            self.external_file_watcher_error = Some(error.clone());
            self.status_message =
                format!("file watcher unavailable; using metadata polling: {error}");
        }
    }

    fn drain_external_file_watcher(&mut self) -> HashSet<PathBuf> {
        let Some(watcher) = self.external_file_watcher.as_ref() else {
            return HashSet::new();
        };
        let drained = watcher.drain();
        if let Some(error) = drained.error {
            self.external_file_watcher = None;
            self.external_file_watcher_error = Some(error.clone());
            self.status_message =
                format!("file watcher unavailable; using metadata polling: {error}");
        }
        drained.changed_paths
    }

    pub(in crate::gui::app::state) fn complete_external_file_check(
        &mut self,
        results: Vec<GuiExternalFileCheckResult>,
    ) {
        self.external_file_check_in_flight = false;
        for result in results {
            self.apply_external_file_check_result(result);
        }
    }

    pub(in crate::gui::app::state) fn apply_external_file_check_result(&mut self, result: GuiExternalFileCheckResult) {
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
                self.replace_tile_document_from_external_change(tile_id, *document);
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
    pub(in crate::gui::app::state) fn poll_external_file_changes(&mut self) {
        let candidates = self.external_file_check_candidates(true);
        let results = check_external_file_changes(candidates);
        for result in results {
            self.apply_external_file_check_result(result);
        }
    }
}
