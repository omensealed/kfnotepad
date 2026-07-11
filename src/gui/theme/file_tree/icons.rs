#[derive(Debug)]
struct GuiTreeIconTheme;

impl IconTheme for GuiTreeIconTheme {
    fn glyph(&self, role: IconRole) -> IconSpec {
        gui_tree_icon_spec(role)
    }
}

pub(super) fn gui_tree_icon_spec(role: IconRole) -> IconSpec {
    let glyph = match role {
        IconRole::FolderClosed => nf::cod::COD_FOLDER,
        IconRole::FolderOpen => nf::cod::COD_FOLDER_OPENED,
        IconRole::File => nf::cod::COD_FILE,
        IconRole::Error => nf::cod::COD_ERROR,
        IconRole::CaretRight => nf::oct::OCT_CHEVRON_RIGHT,
        IconRole::CaretDown => nf::oct::OCT_CHEVRON_DOWN,
        _ => "?",
    };
    IconSpec::new(glyph).with_size(13.0)
}
