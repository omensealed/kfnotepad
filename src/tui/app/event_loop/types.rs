//! Mutable terminal dimensions shared by rendering and input dispatch.

pub(super) struct LoopLayout {
    pub(super) visible_rows: usize,
    pub(super) terminal_width: usize,
    pub(super) gutter_width: usize,
}
