pub(crate) fn gui_editor_scrollbar_model(
    line_count: usize,
    first_line: usize,
    visible_lines: usize,
    track_height: f32,
) -> GuiEditorScrollbarModel {
    let visible_lines = visible_lines.max(1);
    let line_count = line_count.max(1);
    let track_height = track_height.max(1.0);
    let page_delta = visible_lines.min(i32::MAX as usize) as i32;

    if line_count <= visible_lines {
        return GuiEditorScrollbarModel {
            visible: false,
            track_height,
            thumb_top: 0.0,
            thumb_height: track_height,
            page_delta,
            visible_lines,
            line_count,
        };
    }

    let max_first = line_count
        .saturating_sub(visible_lines.saturating_sub(1))
        .max(1);
    let clamped_first = first_line.clamp(1, max_first);
    let proportional_thumb = track_height * (visible_lines as f32 / line_count as f32);
    let thumb_height = proportional_thumb
        .max(GUI_EDITOR_SCROLLBAR_THUMB_MIN_HEIGHT)
        .min(track_height);
    let travel = (track_height - thumb_height).max(0.0);
    let progress = if max_first <= 1 {
        0.0
    } else {
        (clamped_first.saturating_sub(1) as f32 / max_first.saturating_sub(1) as f32)
            .clamp(0.0, 1.0)
    };

    GuiEditorScrollbarModel {
        visible: true,
        track_height,
        thumb_top: travel * progress,
        thumb_height,
        page_delta,
        visible_lines,
        line_count,
    }
}

pub(crate) fn gui_editor_scrollbar_first_line_from_thumb_y(
    model: GuiEditorScrollbarModel,
    y: f32,
    thumb_offset: f32,
) -> usize {
    let line_count = model.line_count.max(1);
    let visible_lines = model.visible_lines.max(1);
    let max_first = line_count
        .saturating_sub(visible_lines.saturating_sub(1))
        .max(1);
    if !model.visible || max_first <= 1 {
        return 1;
    }

    let travel = (model.track_height - model.thumb_height).max(1.0);
    let thumb_top = (y - thumb_offset).clamp(0.0, travel);
    let progress = (thumb_top / travel).clamp(0.0, 1.0);
    1usize
        .saturating_add((progress * max_first.saturating_sub(1) as f32).round() as usize)
        .clamp(1, max_first)
}

pub(crate) fn gui_editor_scrollbar_press_target(
    model: GuiEditorScrollbarModel,
    y: f32,
) -> GuiEditorScrollbarPressTarget {
    if !model.visible {
        return GuiEditorScrollbarPressTarget::None;
    }
    if y < model.thumb_top {
        GuiEditorScrollbarPressTarget::Page(-(model.page_delta.max(1)))
    } else if y <= model.thumb_top + model.thumb_height {
        GuiEditorScrollbarPressTarget::Thumb {
            offset: (y - model.thumb_top).clamp(0.0, model.thumb_height.max(1.0)),
        }
    } else {
        GuiEditorScrollbarPressTarget::Page(model.page_delta.max(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum GuiEditorScrollbarPressTarget {
    None,
    Page(i32),
    Thumb { offset: f32 },
}

impl GuiEditorReplacementSelection {
    pub(crate) fn new(anchor: DocumentCursor, focus: DocumentCursor) -> Option<Self> {
        (anchor != focus).then_some(Self { anchor, focus })
    }

    pub(crate) fn normalized(self) -> (DocumentCursor, DocumentCursor) {
        if document_cursor_is_before_or_equal(self.anchor, self.focus) {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }
}
