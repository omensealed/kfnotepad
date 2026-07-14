//! Canonical editor adapter and render-surface state.

use super::*;

pub(crate) struct GuiEditorAdapter {
    pub(crate) content: text_editor::Content,
    pub(crate) viewport: GuiEditorViewportState,
    pub(crate) viewport_tracks_cursor: bool,
    pub(crate) replacement_selection: Option<GuiEditorReplacementSelection>,
}

pub(crate) struct GuiEditorRenderState {
    pub(crate) line_numbers: GuiEditorLineNumberSnapshot,
    #[cfg(test)]
    pub(crate) viewport_slice: GuiEditorViewportSlice,
}

pub(crate) struct GuiEditorSurfaceModel {
    pub(crate) editor_font: Font,
    pub(crate) editor_size: u32,
    pub(crate) wrapping: Wrapping,
    pub(crate) line_numbers: Option<GuiEditorLineNumberSnapshot>,
    pub(crate) viewport_slice: GuiEditorViewportSlice,
}
