//! Canonical editor adapter and render-surface state.

use super::*;

pub(crate) struct GuiEditorAdapter {
    pub(crate) content: text_editor::Content,
    pub(crate) viewport: GuiEditorViewportState,
    pub(crate) viewport_tracks_cursor: bool,
    pub(crate) replacement_selection: Option<GuiEditorReplacementSelection>,
}

pub(crate) struct GuiEditorRenderState<'a> {
    pub(crate) content: &'a text_editor::Content,
    pub(crate) line_numbers: GuiEditorLineNumberSnapshot,
    #[cfg(test)]
    pub(crate) viewport_slice: GuiEditorViewportSlice,
}

pub(crate) struct GuiEditorSurfaceModel<'a> {
    pub(crate) content: &'a text_editor::Content,
    pub(crate) editor_font: Font,
    pub(crate) editor_size: u32,
    pub(crate) wrapping: Wrapping,
    #[cfg(feature = "syntax")]
    pub(crate) syntax_token: String,
    #[cfg(feature = "syntax")]
    pub(crate) highlighter_theme: highlighter::Theme,
    pub(crate) line_numbers: Option<GuiEditorLineNumberSnapshot>,
    pub(crate) viewport_slice: GuiEditorViewportSlice,
}
