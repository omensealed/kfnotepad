//! Syntax cache, search highlight, and transient IME preedit state.

use super::*;

pub(crate) struct GuiSyntaxCache {
    pub(crate) path: PathBuf,
    pub(crate) line_count: usize,
    pub(crate) highlighted_until: usize,
    pub(crate) lines: Vec<Option<Vec<GuiEditorSyntaxSegment>>>,
    pub(crate) state: Option<SyntaxHighlightCacheState>,
    pub(crate) checkpoints: Vec<GuiSyntaxCheckpoint>,
    #[cfg(test)]
    pub(crate) highlighted_line_operations: usize,
}

pub(crate) struct GuiSyntaxCheckpoint {
    pub(crate) line: usize,
    pub(crate) state: SyntaxHighlightCacheState,
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
