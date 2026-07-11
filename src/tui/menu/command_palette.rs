#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CommandPaletteState {
    pub(crate) query: String,
    pub(crate) selected: usize,
    pub(crate) scroll: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CommandPaletteEntry {
    pub(crate) group: MenuGroup,
    pub(crate) label: &'static str,
    pub(crate) shortcut: Option<&'static str>,
    pub(crate) command: MenuCommand,
}
