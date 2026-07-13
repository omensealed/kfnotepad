//! Editor, UI, search, reader, and restore preference controls.

use super::super::*;

pub(in crate::gui::app::state::view) fn gui_preferences_panel<'a>(
    state: &'a KfnotepadGui,
    panel_tabs: Element<'a, Message>,
    palette: iced::theme::Palette,
) -> Element<'a, Message> {
    widget::column![
        panel_tabs,
        text(LABEL_PREFERENCES).size(gui_ui_heading_text_size(state.settings)),
        gui_tooltip_button(
            format!("Font: {}", state.settings.gui_font_family.display_label()),
            Message::CycleGuiFontFamily,
            "Cycle editor font family",
            state.settings,
        ),
        gui_tooltip_button(
            if cfg!(feature = "syntax") {
                format!("Syntax: {}", state.settings.syntax_theme_id.label())
            } else {
                "Syntax: unavailable".to_string()
            },
            Message::CycleSyntaxTheme,
            if cfg!(feature = "syntax") {
                "Cycle syntax highlighting theme"
            } else {
                "Syntax highlighting unavailable in this build"
            },
            state.settings,
        ),
        row![
            text(format!("Editor size: {}", state.settings.gui_font_size))
                .size(gui_ui_text_size(state.settings)),
            slider(
                MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE,
                state.settings.gui_font_size,
                Message::GuiFontSizeChanged,
            )
            .step(1u16),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            text(format!("UI size: {}", state.settings.gui_ui_font_size))
                .size(gui_ui_text_size(state.settings)),
            slider(
                MIN_GUI_FONT_SIZE..=MAX_GUI_FONT_SIZE,
                state.settings.gui_ui_font_size,
                Message::GuiUiFontSizeChanged,
            )
            .step(1u16),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        checkbox(state.settings.show_line_numbers)
            .label("Line numbers")
            .text_size(gui_ui_text_size(state.settings))
            .spacing(8)
            .style(move |_theme, status| gui_checkbox_style(palette, status))
            .on_toggle(Message::ShowLineNumbersChanged),
        checkbox(state.settings.wrap_lines)
            .label("Wrap text")
            .text_size(gui_ui_text_size(state.settings))
            .spacing(8)
            .style(move |_theme, status| gui_checkbox_style(palette, status))
            .on_toggle(Message::WrapLinesChanged),
        checkbox(state.settings.search_case_sensitive)
            .label("Case-sensitive search")
            .text_size(gui_ui_text_size(state.settings))
            .spacing(8)
            .style(move |_theme, status| gui_checkbox_style(palette, status))
            .on_toggle(Message::SearchCaseSensitiveChanged),
        checkbox(state.settings.gui_reader_mode_enabled)
            .label("Reader mode")
            .text_size(gui_ui_text_size(state.settings))
            .spacing(8)
            .style(move |_theme, status| gui_checkbox_style(palette, status))
            .on_toggle(Message::ReaderModeChanged),
        row![
            text(format!(
                "Reader speed: {} lpm",
                state.settings.gui_reader_lines_per_minute
            ))
            .size(gui_ui_text_size(state.settings)),
            slider(
                MIN_GUI_READER_LINES_PER_MINUTE..=MAX_GUI_READER_LINES_PER_MINUTE,
                state.settings.gui_reader_lines_per_minute,
                Message::ReaderSpeedChanged,
            )
            .step(5u16),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        checkbox(state.settings.gui_restore_last_workspace)
            .label("Restore last workspace")
            .text_size(gui_ui_text_size(state.settings))
            .spacing(8)
            .style(move |_theme, status| gui_checkbox_style(palette, status))
            .on_toggle(Message::RestoreLastWorkspaceChanged),
    ]
    .spacing(5)
    .into()
}
