#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiImeInputMethodRequest {
    pub(crate) visual_row: usize,
    pub(crate) cursor_column: usize,
    pub(crate) gutter_width: f32,
    pub(crate) character_width: f32,
    pub(crate) row_height: f32,
    pub(crate) preedit: Option<input_method::Preedit<String>>,
}

impl GuiImeInputMethodRequest {
    pub(crate) fn cursor_rect(&self, bounds: Rectangle) -> Rectangle {
        Rectangle::new(
            iced::Point::new(
                bounds.x + self.gutter_width + self.cursor_column as f32 * self.character_width,
                bounds.y + self.visual_row as f32 * self.row_height,
            ),
            Size::new(1.0, self.row_height),
        )
    }
}
