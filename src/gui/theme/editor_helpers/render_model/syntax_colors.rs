use super::*;

pub(in crate::gui::app::state) fn gui_editor_line_syntax_colors(
    line: &GuiEditorViewportLine,
) -> Vec<Option<Color>> {
    let mut colors = Vec::new();
    if let Some(segments) = &line.syntax_segments {
        for segment in segments {
            colors.extend(segment.text.chars().map(|_| Some(segment.color)));
        }
    }
    colors.resize(line.text.chars().count(), None);
    colors
}

pub(in crate::gui::app::state) fn gui_editor_push_read_only_segment(
    segments: &mut Vec<GuiEditorReadOnlyLineSegment>,
    text: &mut String,
    selected: bool,
    syntax_color: Option<Color>,
) {
    if text.is_empty() {
        return;
    }
    segments.push(GuiEditorReadOnlyLineSegment {
        text: std::mem::take(text),
        selected,
        syntax_color,
    });
}
