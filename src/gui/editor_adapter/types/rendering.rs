#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GuiEditorViewportState {
    pub(crate) first_line: usize,
    pub(crate) visible_lines: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiEditorViewportLine {
    pub(crate) number: usize,
    pub(crate) text: String,
    pub(crate) cursor_column: Option<usize>,
    pub(crate) selection: Option<GuiEditorSelectionSpan>,
    pub(crate) syntax_segments: Option<Vec<GuiEditorSyntaxSegment>>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiEditorViewportSlice {
    pub(crate) line_count: usize,
    pub(crate) first_line: usize,
    pub(crate) lines: Vec<GuiEditorViewportLine>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuiEditorReadOnlyRenderModel {
    pub(crate) line_count: usize,
    pub(crate) first_line: usize,
    pub(crate) gutter_text: String,
    pub(crate) body_text: String,
    pub(crate) cursor_row_in_view: Option<usize>,
    pub(crate) cursor_column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiEditorReadOnlyLineSegment {
    pub(crate) text: String,
    pub(crate) selected: bool,
    pub(crate) syntax_color: Option<Color>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiEditorReadOnlyVisualRow {
    pub(crate) line: GuiEditorViewportLine,
    pub(crate) viewport_row: usize,
    pub(crate) source_column_start: usize,
    pub(crate) show_line_number: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GuiEditorScrollbarModel {
    pub(crate) visible: bool,
    pub(crate) track_height: f32,
    pub(crate) thumb_top: f32,
    pub(crate) thumb_height: f32,
    pub(crate) page_delta: i32,
    pub(crate) visible_lines: usize,
    pub(crate) line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GuiEditorSyntaxSegment {
    pub(crate) text: String,
    pub(crate) color: Color,
}
