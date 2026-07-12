//! Menu group identity and active selection state.

pub(crate) const MENU_GROUPS: [MenuGroup; 7] = [
    MenuGroup::File,
    MenuGroup::Edit,
    MenuGroup::View,
    MenuGroup::Go,
    MenuGroup::Tabs,
    MenuGroup::Workspace,
    MenuGroup::Help,
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MenuState {
    pub(crate) group: MenuGroup,
    pub(crate) selected: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum MenuGroup {
    #[default]
    File,
    Edit,
    View,
    Go,
    Tabs,
    Workspace,
    Help,
}
