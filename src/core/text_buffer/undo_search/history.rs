impl TextBuffer {
    fn record_undo_for_typed_insert(&mut self, row: usize, column: usize) {
        let now = Instant::now();
        let can_merge = self.insert_undo_group.is_some_and(|group| {
            group.row == row
                && group.next_column == column
                && now.duration_since(group.last_edit) <= TYPING_UNDO_COALESCE_WINDOW
        });

        if can_merge {
            self.insert_undo_group = Some(InsertUndoGroup {
                row,
                next_column: column.saturating_add(1),
                last_edit: now,
            });
            return;
        }

        self.record_undo();
        self.insert_undo_group = Some(InsertUndoGroup {
            row,
            next_column: column.saturating_add(1),
            last_edit: now,
        });
    }

    pub(crate) fn break_undo_group(&mut self) {
        self.insert_undo_group = None;
    }

    pub(crate) fn begin_compound_edit(&mut self) {
        self.compound_edit = match std::mem::replace(
            &mut self.compound_edit,
            CompoundEditState::Inactive,
        ) {
            CompoundEditState::Inactive => {
                self.break_undo_group();
                CompoundEditState::Pending {
                    depth: 1,
                    snapshot: Box::new(BufferSnapshot::from_buffer(self)),
                }
            }
            CompoundEditState::Pending { depth, snapshot } => CompoundEditState::Pending {
                depth: depth.saturating_add(1),
                snapshot,
            },
            CompoundEditState::Recorded { depth } => CompoundEditState::Recorded {
                depth: depth.saturating_add(1),
            },
        };
    }

    pub(crate) fn end_compound_edit(&mut self) {
        self.compound_edit = match std::mem::replace(
            &mut self.compound_edit,
            CompoundEditState::Inactive,
        ) {
            CompoundEditState::Pending { depth, snapshot } if depth > 1 => {
                CompoundEditState::Pending {
                    depth: depth - 1,
                    snapshot,
                }
            }
            CompoundEditState::Recorded { depth } if depth > 1 => {
                CompoundEditState::Recorded { depth: depth - 1 }
            }
            _ => {
                self.break_undo_group();
                CompoundEditState::Inactive
            }
        };
        if matches!(self.compound_edit, CompoundEditState::Inactive) {
            self.break_undo_group();
        }
    }

    fn mark_changed(&mut self) {
        self.dirty = true;
        self.edit_revision = self.edit_revision.wrapping_add(1);
    }

    fn record_undo(&mut self) {
        self.insert_undo_group = None;
        let snapshot = match std::mem::replace(
            &mut self.compound_edit,
            CompoundEditState::Inactive,
        ) {
            CompoundEditState::Pending { depth, snapshot } => {
                self.compound_edit = CompoundEditState::Recorded { depth };
                *snapshot
            }
            recorded @ CompoundEditState::Recorded { .. } => {
                self.compound_edit = recorded;
                return;
            }
            CompoundEditState::Inactive => BufferSnapshot::from_buffer(self),
        };
        push_history_snapshot(
            &mut self.undo_history,
            &mut self.undo_bytes,
            snapshot,
            MAX_UNDO_HISTORY,
            MAX_UNDO_BYTES,
        );
        self.redo_history.clear();
        self.redo_bytes = 0;
    }
}
