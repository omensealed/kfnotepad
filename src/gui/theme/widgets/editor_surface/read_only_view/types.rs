use super::*;

#[derive(Clone, Copy)]
pub(super) struct GuiReadOnlyLineRowContext {
    pub(super) pane: pane_grid::Pane,
    pub(super) line_number_width: Option<f32>,
    pub(super) gutter_width: f32,
    pub(super) character_width: f32,
    pub(super) row_height: f32,
    pub(super) editor_font: Font,
    pub(super) editor_size: u32,
    pub(super) settings: EditorSettings,
    pub(super) palette: iced::theme::Palette,
    pub(super) search_highlight_active: bool,
}

pub(super) struct GuiReadOnlyBodyContext {
    pub(super) pane: pane_grid::Pane,
    pub(super) visual_rows: Vec<GuiEditorReadOnlyVisualRow>,
    pub(super) body_columns: usize,
    pub(super) visible_row_budget: usize,
    pub(super) gutter_width: f32,
    pub(super) surface_height: f32,
    pub(super) settings: EditorSettings,
    pub(super) palette: iced::theme::Palette,
}
