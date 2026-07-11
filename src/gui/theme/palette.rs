pub(super) fn gui_theme(theme_id: EditorThemeId) -> Theme {
    Theme::custom(
        format!("kfnotepad {}", theme_id.label()),
        gui_theme_palette(theme_id),
    )
}

pub(super) fn gui_theme_palette(theme_id: EditorThemeId) -> iced::theme::Palette {
    match theme_id {
        EditorThemeId::Nocturne => iced::theme::Palette {
            background: color(10, 12, 24),
            text: color(226, 232, 246),
            primary: color(92, 119, 255),
            success: color(56, 189, 126),
            warning: color(244, 202, 94),
            danger: color(255, 91, 112),
        },
        EditorThemeId::Aurora => iced::theme::Palette {
            background: color(6, 20, 24),
            text: color(224, 252, 241),
            primary: color(35, 211, 171),
            success: color(111, 232, 123),
            warning: color(255, 218, 92),
            danger: color(255, 99, 132),
        },
        EditorThemeId::Paper => iced::theme::Palette {
            background: color(245, 226, 244),
            text: color(34, 24, 48),
            primary: color(118, 67, 169),
            success: color(35, 128, 105),
            warning: color(139, 83, 31),
            danger: color(155, 48, 96),
        },
        EditorThemeId::Terminal => iced::theme::Palette {
            background: color(0, 18, 7),
            text: color(177, 255, 177),
            primary: color(72, 255, 112),
            success: color(44, 220, 96),
            warning: color(255, 228, 92),
            danger: color(255, 92, 92),
        },
        EditorThemeId::Abyss => iced::theme::Palette {
            background: color(3, 7, 18),
            text: color(206, 240, 255),
            primary: color(102, 229, 255),
            success: color(79, 209, 197),
            warning: color(252, 211, 77),
            danger: color(255, 64, 96),
        },
        EditorThemeId::Terror => iced::theme::Palette {
            background: color(24, 0, 30),
            text: color(255, 188, 236),
            primary: color(255, 42, 160),
            success: color(112, 255, 128),
            warning: color(255, 238, 70),
            danger: color(255, 58, 58),
        },
    }
}

pub(super) fn gui_highlighter_theme(theme_id: EditorThemeId) -> highlighter::Theme {
    match theme_id {
        EditorThemeId::Paper => highlighter::Theme::InspiredGitHub,
        EditorThemeId::Terminal => highlighter::Theme::Base16Mocha,
        EditorThemeId::Abyss | EditorThemeId::Nocturne => highlighter::Theme::Base16Ocean,
        EditorThemeId::Aurora => highlighter::Theme::SolarizedDark,
        EditorThemeId::Terror => highlighter::Theme::Base16Eighties,
    }
}

pub(super) fn gui_editor_font(font_family: GuiFontFamily) -> Font {
    match font_family {
        GuiFontFamily::Monospace => Font::MONOSPACE,
        GuiFontFamily::SansSerif => Font::DEFAULT,
        GuiFontFamily::Serif => Font {
            family: iced::font::Family::Serif,
            ..Font::DEFAULT
        },
        GuiFontFamily::JetBrainsMono => Font::with_name("JetBrains Mono"),
        GuiFontFamily::FiraCode => Font::with_name("Fira Code"),
    }
}

pub(super) fn gui_editor_wrapping(wrap_lines: bool) -> Wrapping {
    if wrap_lines {
        Wrapping::WordOrGlyph
    } else {
        Wrapping::None
    }
}

pub(super) fn gui_editor_effective_wrapping(wrap_lines: bool, show_line_numbers: bool) -> Wrapping {
    let _ = show_line_numbers;
    gui_editor_wrapping(wrap_lines)
}

pub(super) fn gui_editor_surface_model<'a>(
    settings: EditorSettings,
    document: &TextDocument,
    editor: &'a GuiEditorAdapter,
    syntax_highlighter: &SyntaxHighlighter,
    syntax_cache: Option<&GuiSyntaxCache>,
) -> GuiEditorSurfaceModel<'a> {
    let render_state =
        editor.render_state(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, settings.gui_font_size);
    let render_viewport_slice = editor
        .render_viewport_slice_from_lines(document.buffer.lines(), GUI_EDITOR_RENDER_LINE_BUDGET);
    let viewport_slice =
        gui_editor_viewport_slice_with_cached_syntax(render_viewport_slice, syntax_cache);
    GuiEditorSurfaceModel {
        content: render_state.content,
        editor_font: gui_editor_font(settings.gui_font_family),
        editor_size: u32::from(settings.gui_font_size),
        wrapping: gui_editor_effective_wrapping(settings.wrap_lines, settings.show_line_numbers),
        syntax_token: syntax_highlighter.syntax_token_for_document(document),
        highlighter_theme: gui_highlighter_theme(settings.syntax_theme_id),
        line_numbers: settings
            .show_line_numbers
            .then_some(render_state.line_numbers),
        viewport_slice,
    }
}

pub(super) fn gui_ui_font_size(settings: EditorSettings) -> u32 {
    settings.gui_ui_font_size.into()
}

pub(super) fn gui_ui_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
}

pub(super) fn gui_ui_small_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_sub(2)
        .max(MIN_GUI_FONT_SIZE.into())
}

pub(super) fn gui_ui_heading_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_add(4)
        .min(MAX_GUI_FONT_SIZE.into())
}

pub(super) fn gui_ui_icon_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_add(1)
        .min(MAX_GUI_FONT_SIZE.into())
}

pub(super) fn gui_ui_tooltip_text_size(settings: EditorSettings) -> u32 {
    gui_ui_font_size(settings)
        .saturating_sub(2)
        .max(MIN_GUI_FONT_SIZE.into())
}

pub(super) fn gui_line_number_gutter_text(
    first_line: usize,
    line_count: usize,
    visible_lines: usize,
) -> String {
    let total = line_count.max(1);
    let start = first_line.clamp(1, total);
    let end = (start + visible_lines.saturating_sub(1)).min(total);

    (start..=end)
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn gui_line_number_gutter_width(line_count: usize, editor_font_size: u16) -> f32 {
    let digits = line_count.max(1).to_string().len() as f32;
    let digit_width = f32::from(editor_font_size) * 0.62;
    GUI_LINE_NUMBER_GUTTER_HORIZONTAL_PADDING + digits * digit_width
}
