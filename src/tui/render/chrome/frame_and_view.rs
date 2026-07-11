pub(crate) fn compose_tab_label(index: usize, item: &TabStripItem) -> String {
    let dirty = if item.dirty { "*" } else { "" };
    format!(" {}:{}{} ", index + 1, item.label, dirty)
}

#[derive(Clone, Copy)]
pub(crate) struct RenderFrame {
    pub(crate) theme: EditorTheme,
    pub(crate) gutter_width: usize,
    pub(crate) terminal_width: usize,
    pub(crate) origin_column: u16,
    pub(crate) body_top: u16,
    pub(crate) no_color: bool,
}

impl RenderFrame {
    fn move_to(self, column: u16, row: u16) -> MoveTo {
        MoveTo(self.origin_column.saturating_add(column), row)
    }
}

pub(crate) fn queue_set_foreground_color(
    writer: &mut impl Write,
    frame: &RenderFrame,
    color: Color,
) -> io::Result<()> {
    if frame.no_color {
        return Ok(());
    }
    queue!(writer, SetForegroundColor(color))
}

pub(crate) fn queue_set_background_color(
    writer: &mut impl Write,
    frame: &RenderFrame,
    color: Color,
) -> io::Result<()> {
    if frame.no_color {
        return Ok(());
    }
    queue!(writer, SetBackgroundColor(color))
}

#[derive(Clone, Copy)]
pub(crate) struct EditorView<'a> {
    pub(crate) cursor: Cursor,
    pub(crate) viewport_start: usize,
    pub(crate) horizontal_offset: usize,
    pub(crate) visible_rows: usize,
    pub(crate) status: &'a str,
    pub(crate) settings: EditorSettings,
    pub(crate) menu: Option<MenuState>,
    pub(crate) sidebar_width: usize,
    pub(crate) tab_strip: &'a [TabStripItem],
    pub(crate) search_highlight: Option<SearchHighlightView<'a>>,
}

#[derive(Clone, Copy)]
pub(crate) struct SearchHighlightView<'a> {
    pub(crate) query: &'a str,
    pub(crate) mode: SearchMode,
}

struct EditorLineView<'a> {
    screen_row: u16,
    document_row: usize,
    line: &'a str,
    settings: EditorSettings,
    horizontal_offset: usize,
    highlighted_line: Option<Vec<(SyntectStyle, String)>>,
    search_highlight: Option<SearchHighlightView<'a>>,
}
