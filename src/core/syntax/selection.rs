//! File-path based syntax selection and display metadata.

use std::path::Path;

use syntect::parsing::SyntaxReference;

use super::SyntaxHighlighter;
use crate::core::TextDocument;

impl SyntaxHighlighter {
    pub(super) fn syntax_for_path(&self, path: &Path) -> &SyntaxReference {
        self.syntax_set
            .find_syntax_for_file(path)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }

    pub(super) fn syntax_for_document(&self, document: &TextDocument) -> &SyntaxReference {
        self.syntax_for_path(&document.path)
    }

    pub fn syntax_name_for_document(&self, document: &TextDocument) -> &str {
        self.syntax_for_document(document).name.as_str()
    }

    pub fn syntax_token_for_document(&self, document: &TextDocument) -> String {
        let syntax = self.syntax_for_document(document);
        syntax
            .file_extensions
            .first()
            .cloned()
            .unwrap_or_else(|| syntax.name.clone())
    }
}
