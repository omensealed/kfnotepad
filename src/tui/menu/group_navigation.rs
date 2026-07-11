impl MenuGroup {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Go => "Go",
            Self::Tabs => "Tabs",
            Self::Workspace => "Workspace",
            Self::Help => "Help",
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::File => Self::Help,
            Self::Edit => Self::File,
            Self::View => Self::Edit,
            Self::Go => Self::View,
            Self::Tabs => Self::Go,
            Self::Workspace => Self::Tabs,
            Self::Help => Self::Workspace,
        }
    }

    pub(crate) fn next(self) -> Self {
        match self {
            Self::File => Self::Edit,
            Self::Edit => Self::View,
            Self::View => Self::Go,
            Self::Go => Self::Tabs,
            Self::Tabs => Self::Workspace,
            Self::Workspace => Self::Help,
            Self::Help => Self::File,
        }
    }
}
