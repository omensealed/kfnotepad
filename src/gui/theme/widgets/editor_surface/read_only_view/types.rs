#[derive(Clone, Copy)]
struct GuiReadOnlyLineRowContext {
    pane: pane_grid::Pane,
    line_number_width: Option<f32>,
    gutter_width: f32,
    character_width: f32,
    row_height: f32,
    editor_font: Font,
    editor_size: u32,
    settings: EditorSettings,
    palette: iced::theme::Palette,
    search_highlight_active: bool,
}

struct GuiReadOnlyBodyContext {
    pane: pane_grid::Pane,
    source_lines: Vec<GuiEditorViewportLine>,
    first_line: usize,
    wrapping: Wrapping,
    body_columns: usize,
    visible_row_budget: usize,
    gutter_width: f32,
    surface_height: f32,
    settings: EditorSettings,
    palette: iced::theme::Palette,
}
