pub enum EditorTabDocument<'a> {
    Borrowed(&'a mut TextDocument),
    Owned(Box<TextDocument>),
}

impl AsRef<TextDocument> for EditorTabDocument<'_> {
    fn as_ref(&self) -> &TextDocument {
        match self {
            Self::Borrowed(document) => document,
            Self::Owned(document) => document,
        }
    }
}

impl AsMut<TextDocument> for EditorTabDocument<'_> {
    fn as_mut(&mut self) -> &mut TextDocument {
        match self {
            Self::Borrowed(document) => document,
            Self::Owned(document) => document,
        }
    }
}

pub struct EditorTab<'a> {
    pub document: EditorTabDocument<'a>,
    pub state: EditorTabState,
}

pub struct EditorWorkspace<'a> {
    pub tabs: Vec<EditorTab<'a>>,
    pub active: usize,
}
