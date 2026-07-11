#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GuiFontFamily {
    #[default]
    Monospace,
    SansSerif,
    Serif,
    JetBrainsMono,
    FiraCode,
}

impl GuiFontFamily {
    pub const ALL: [Self; 5] = [
        Self::Monospace,
        Self::SansSerif,
        Self::Serif,
        Self::JetBrainsMono,
        Self::FiraCode,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Monospace => "monospace",
            Self::SansSerif => "sans-serif",
            Self::Serif => "serif",
            Self::JetBrainsMono => "jetbrains-mono",
            Self::FiraCode => "fira-code",
        }
    }

    pub fn display_label(self) -> &'static str {
        match self {
            Self::Monospace => "Monospace",
            Self::SansSerif => "Sans serif",
            Self::Serif => "Serif",
            Self::JetBrainsMono => "JetBrains Mono",
            Self::FiraCode => "Fira Code",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "monospace" => Some(Self::Monospace),
            "sans-serif" => Some(Self::SansSerif),
            "serif" => Some(Self::Serif),
            "jetbrains-mono" => Some(Self::JetBrainsMono),
            "fira-code" => Some(Self::FiraCode),
            _ => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Monospace => Self::SansSerif,
            Self::SansSerif => Self::Serif,
            Self::Serif => Self::JetBrainsMono,
            Self::JetBrainsMono => Self::FiraCode,
            Self::FiraCode => Self::Monospace,
        }
    }
}
