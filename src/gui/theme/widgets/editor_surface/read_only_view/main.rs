use super::*;

pub(in crate::gui::app::state) fn gui_editor_read_only_view(
    pane: pane_grid::Pane,
    editor_surface: &GuiEditorSurfaceModel<'_>,
    settings: EditorSettings,
    search_highlight_active: bool,
    ime_preedit: Option<GuiImePreedit>,
) -> Element<'static, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    let line_number_width = editor_surface
        .line_numbers
        .as_ref()
        .map(|line_numbers| line_numbers.width);
    let source_lines = editor_surface.viewport_slice.lines.clone();
    let first_line = editor_surface.viewport_slice.first_line;
    let line_count = editor_surface.viewport_slice.line_count;
    let wrapping = editor_surface.wrapping;
    let editor_font = editor_surface.editor_font;
    let editor_size = editor_surface.editor_size;

    responsive(move |surface_size| {
        let gutter_width = line_number_width.unwrap_or_default()
            + line_number_width
                .map(|_| GUI_LINE_NUMBER_SEPARATOR_WIDTH)
                .unwrap_or_default();
        let body_width = (surface_size.width - gutter_width).max(1.0);
        let character_width = gui_editor_replacement_character_width(settings);
        let row_height = gui_editor_replacement_row_height(settings);
        let body_columns = (body_width / character_width).floor().max(1.0) as usize;
        let visible_row_budget = gui_editor_visible_row_budget(surface_size.height, row_height);
        let scrollbar_model = gui_editor_scrollbar_model(
            line_count,
            first_line,
            visible_row_budget,
            surface_size.height,
        );
        let visual_rows =
            gui_editor_read_only_visual_rows(&source_lines, first_line, wrapping, body_columns)
                .into_iter()
                .take(visible_row_budget);
        let mut editor_rows = iced::widget::Column::new()
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Shrink);
        let mut ime_request = None;
        let line_row_context = GuiReadOnlyLineRowContext {
            pane,
            line_number_width,
            gutter_width,
            character_width,
            row_height,
            editor_font,
            editor_size,
            settings,
            palette,
            search_highlight_active,
        };

        for (rendered_row, visual_row) in visual_rows.enumerate() {
            let (line_row, row_ime_request) = gui_editor_read_only_line_row(
                line_row_context,
                visual_row,
                rendered_row,
                ime_preedit.as_ref(),
            );
            if row_ime_request.is_some() {
                ime_request = row_ime_request;
            }
            editor_rows = editor_rows.push(line_row);
        }

        let editor_body = gui_editor_read_only_body(
            editor_rows,
            GuiReadOnlyBodyContext {
                pane,
                source_lines: source_lines.clone(),
                first_line,
                wrapping,
                body_columns,
                visible_row_budget,
                gutter_width,
                surface_height: surface_size.height,
                settings,
                palette,
            },
        );

        let editor_with_scrollbar: Element<'static, Message> = row![
            editor_body,
            gui_editor_scrollbar_view(pane, scrollbar_model, palette, settings)
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        Element::new(GuiInputMethodArea::new(editor_with_scrollbar, ime_request))
    })
    .into()
}
