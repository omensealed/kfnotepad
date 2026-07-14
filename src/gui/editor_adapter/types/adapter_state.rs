//! Canonical editor adapter and render-surface state.

use super::*;

pub(crate) struct GuiEditorAdapter {
    pub(crate) cursor: DocumentCursor,
    pub(crate) line_count: usize,
    pub(crate) viewport: GuiEditorViewportState,
    pub(crate) viewport_tracks_cursor: bool,
    pub(crate) replacement_selection: Option<GuiEditorReplacementSelection>,
    pub(crate) visual_layout_cache: std::sync::Arc<std::sync::Mutex<GuiEditorVisualLayoutCache>>,
}

pub(crate) struct GuiEditorRenderState {
    pub(crate) line_numbers: GuiEditorLineNumberSnapshot,
}

pub(crate) struct GuiEditorSurfaceModel {
    pub(crate) editor_font: Font,
    pub(crate) editor_size: u32,
    pub(crate) wrapping: Wrapping,
    pub(crate) line_numbers: Option<GuiEditorLineNumberSnapshot>,
    pub(crate) viewport_slice: GuiEditorViewportSlice,
    pub(crate) document_revision: u64,
    pub(crate) visual_layout_cache: std::sync::Arc<std::sync::Mutex<GuiEditorVisualLayoutCache>>,
}
