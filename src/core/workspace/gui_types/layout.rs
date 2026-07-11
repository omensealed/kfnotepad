#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GuiTileId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiTileSaveStatus {
    Saved,
    Modified,
    SaveFailed { message: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiSplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiTileMoveDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiTileResizeDirection {
    Wider,
    Narrower,
    Taller,
    Shorter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiTileLayoutIntent {
    Split {
        tile_id: GuiTileId,
        direction: GuiSplitDirection,
    },
    Move {
        tile_id: GuiTileId,
        direction: GuiTileMoveDirection,
    },
    Resize {
        tile_id: GuiTileId,
        direction: GuiTileResizeDirection,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuiLayoutAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuiLayoutNode {
    Leaf {
        ordinal: usize,
    },
    Split {
        axis: GuiLayoutAxis,
        ratio_per_mille: u16,
        first: Box<GuiLayoutNode>,
        second: Box<GuiLayoutNode>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuiLayout {
    pub browser_visible: bool,
    pub browser_width_px: Option<u16>,
    pub root: GuiLayoutNode,
    pub minimized_ordinals: Vec<usize>,
}
