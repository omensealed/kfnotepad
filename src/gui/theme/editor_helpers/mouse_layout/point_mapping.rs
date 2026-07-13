use super::*;

#[cfg(test)]
pub(in crate::gui::app::state) fn gui_editor_replacement_mouse_point_from_line_point(
    point: iced::Point,
    viewport_row: usize,
    settings: EditorSettings,
    body_width: f32,
    wrapping: Wrapping,
) -> GuiEditorReplacementMousePoint {
    let character_width = (f32::from(settings.gui_font_size) * 0.62).max(1.0);
    let column_in_visual_row = (point.x.max(0.0) / character_width).floor() as usize;
    let visual_row = if wrapping == Wrapping::None {
        0
    } else {
        let line_height = (f32::from(settings.gui_font_size) * GUI_EDITOR_LINE_HEIGHT).max(1.0);
        (point.y.max(0.0) / line_height).floor() as usize
    };
    let visual_row_columns = if wrapping == Wrapping::None {
        0
    } else {
        (body_width.max(character_width) / character_width)
            .floor()
            .max(1.0) as usize
    };

    GuiEditorReplacementMousePoint {
        viewport_row,
        column: visual_row
            .saturating_mul(visual_row_columns)
            .saturating_add(column_in_visual_row),
    }
}

pub(in crate::gui::app::state) fn gui_editor_replacement_mouse_point_from_visual_row_point(
    point: iced::Point,
    viewport_row: usize,
    source_column_start: usize,
    visual_row_text: &str,
    settings: EditorSettings,
) -> GuiEditorReplacementMousePoint {
    let column_in_visual_row = gui_editor_char_column_from_pixel_x(
        visual_row_text,
        point.x,
        gui_editor_replacement_character_width(settings),
    );
    GuiEditorReplacementMousePoint {
        viewport_row,
        column: source_column_start.saturating_add(column_in_visual_row),
    }
}

pub(in crate::gui::app::state) fn gui_editor_replacement_mouse_point_from_body_point(
    point: iced::Point,
    source_lines: &[GuiEditorViewportLine],
    first_line: usize,
    wrapping: Wrapping,
    hit_test: GuiEditorBodyHitTest,
    settings: EditorSettings,
) -> GuiEditorReplacementMousePoint {
    let text_point = iced::Point::new(point.x - hit_test.text_origin_x, point.y);
    let row_height = gui_editor_replacement_row_height(settings);
    let target_visual_row = (point.y.max(0.0) / row_height).floor() as usize;
    let visual_rows =
        gui_editor_read_only_visual_rows(source_lines, first_line, wrapping, hit_test.columns)
            .into_iter()
            .take(hit_test.visible_rows.max(1))
            .collect::<Vec<_>>();

    if let Some(visual_row) =
        visual_rows.get(target_visual_row.min(visual_rows.len().saturating_sub(1)))
    {
        return gui_editor_replacement_mouse_point_from_visual_row_point(
            text_point,
            visual_row.viewport_row,
            visual_row.source_column_start,
            &visual_row.line.text,
            settings,
        );
    }

    let display_column =
        (text_point.x.max(0.0) / gui_editor_replacement_character_width(settings)).floor() as usize;
    GuiEditorReplacementMousePoint {
        viewport_row: target_visual_row.min(hit_test.visible_rows.saturating_sub(1)),
        column: display_column,
    }
}

pub(in crate::gui::app::state) fn gui_editor_drag_edge_from_body_point(
    pane: pane_grid::Pane,
    point: iced::Point,
    surface_height: f32,
    point_column: usize,
    settings: EditorSettings,
) -> GuiEditorDragEdge {
    let edge_zone = gui_editor_replacement_row_height(settings).max(1.0);
    let direction = if point.y <= edge_zone {
        -1
    } else if point.y >= (surface_height - edge_zone).max(edge_zone) {
        1
    } else {
        0
    };
    GuiEditorDragEdge {
        pane,
        direction,
        column: point_column,
    }
}
