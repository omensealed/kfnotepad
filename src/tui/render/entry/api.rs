#[cfg(test)]
pub(crate) fn render_editor(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
) -> io::Result<()> {
    render_editor_with_width_and_color(
        writer,
        document,
        view,
        highlighter,
        terminal_width(),
        no_color_enabled(),
    )
}

pub(crate) fn render_editor_with_cache(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    syntax_cache: &mut TuiSyntaxHighlightCache,
) -> io::Result<()> {
    render_editor_with_width_color_and_cache(
        writer,
        document,
        view,
        highlighter,
        terminal_width(),
        no_color_enabled(),
        Some(syntax_cache),
    )
}

#[cfg(test)]
pub(crate) fn render_editor_with_width(
    writer: &mut impl Write,
    document: &TextDocument,
    view: EditorView<'_>,
    highlighter: &SyntaxHighlighter,
    terminal_width: usize,
) -> io::Result<()> {
    render_editor_with_width_and_color(
        writer,
        document,
        view,
        highlighter,
        terminal_width,
        no_color_enabled(),
    )
}
