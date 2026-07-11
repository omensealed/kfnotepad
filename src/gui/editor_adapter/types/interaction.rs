#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GuiEditorDragState {
    pub(crate) pane: pane_grid::Pane,
    pub(crate) anchor: DocumentCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GuiEditorDragEdge {
    pub(crate) pane: pane_grid::Pane,
    pub(crate) direction: i32,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GuiEditorScrollbarDrag {
    pub(crate) pane: pane_grid::Pane,
    pub(crate) thumb_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GuiEditorBodyHitTest {
    pub(crate) columns: usize,
    pub(crate) visible_rows: usize,
    pub(crate) text_origin_x: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GuiEditorSelectionSpan {
    pub(crate) start_column: usize,
    pub(crate) end_column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GuiEditorReplacementSelection {
    pub(crate) anchor: DocumentCursor,
    pub(crate) focus: DocumentCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GuiEditorReplacementMousePoint {
    pub(crate) viewport_row: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuiEditorReplacementInput {
    InsertChar(char),
    InsertNewline,
    DeleteBackward,
    DeleteForward,
    DeletePreviousWord,
    DeleteNextWord,
    DeleteToLineEnd,
    Move(kfnotepad::CursorMove),
    MoveLineStart,
    MoveLineEnd,
    ScrollViewportLines(i32),
    SelectAll,
    #[allow(dead_code)]
    SelectRange {
        anchor: DocumentCursor,
        focus: DocumentCursor,
    },
    ClearSelection,
}
