pub(crate) struct GuiSyntaxCache {
    pub(crate) path: PathBuf,
    pub(crate) line_count: usize,
    pub(crate) highlighted_until: usize,
    pub(crate) lines: Vec<Option<Vec<GuiEditorSyntaxSegment>>>,
    pub(crate) state: Option<SyntaxHighlightCacheState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuiSearchHighlight {
    pub(crate) tile_id: GuiTileId,
    pub(crate) query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GuiImePreedit {
    pub(crate) tile_id: GuiTileId,
    pub(crate) content: String,
    pub(crate) selection: Option<Range<usize>>,
}
