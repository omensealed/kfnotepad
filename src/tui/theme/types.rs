#[derive(Clone, Copy)]
pub(crate) struct EditorTheme {
    pub(crate) header_fg: Color,
    pub(crate) header_bg: Color,
    pub(crate) gutter_fg: Color,
    pub(crate) status_fg: Color,
    pub(crate) status_bg: Color,
    pub(crate) search_fg: Color,
    pub(crate) search_bg: Color,
    pub(crate) help_fg: Color,
    pub(crate) help_bg: Color,
    pub(crate) dirty_fg: Color,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self::for_id(EditorThemeId::Nocturne)
    }
}
