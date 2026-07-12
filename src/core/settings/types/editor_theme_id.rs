//! Stable built-in editor theme identifiers and labels.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorThemeId {
    #[default]
    Nocturne,
    Aurora,
    Paper,
    Terminal,
    Abyss,
    Terror,
}

impl EditorThemeId {
    pub fn next(self) -> Self {
        match self {
            Self::Nocturne => Self::Aurora,
            Self::Aurora => Self::Paper,
            Self::Paper => Self::Terminal,
            Self::Terminal => Self::Abyss,
            Self::Abyss => Self::Terror,
            Self::Terror => Self::Nocturne,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Nocturne => "nocturne",
            Self::Aurora => "aurora",
            Self::Paper => "pastel",
            Self::Terminal => "terminal",
            Self::Abyss => "abyss",
            Self::Terror => "terror",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "nocturne" => Some(Self::Nocturne),
            "aurora" => Some(Self::Aurora),
            "paper" | "pastel" => Some(Self::Paper),
            "terminal" => Some(Self::Terminal),
            "abyss" => Some(Self::Abyss),
            "terror" => Some(Self::Terror),
            _ => None,
        }
    }
}
