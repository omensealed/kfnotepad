//! Application header rendering and responsive menu visibility.

use super::*;

pub(crate) fn write_header(
    writer: &mut impl Write,
    document: &TextDocument,
    menu: Option<MenuState>,
    frame: RenderFrame,
) -> io::Result<()> {
    let state = if document.buffer.is_dirty() {
        " modified "
    } else {
        " saved "
    };
    let label = " kfnotepad ";
    let show_menu_bar = show_menu_bar(frame);
    let reserved = text_display_width(label)
        + text_display_width(state)
        + if show_menu_bar {
            text_display_width(menu_bar_text())
        } else {
            0
        };
    let path_width = frame.terminal_width.saturating_sub(reserved).max(1);
    let path = fit_text_end(&format!(" {} ", document.path.display()), path_width);

    let mut remaining = frame.terminal_width;
    queue!(
        writer,
        frame.move_to(0, 0),
        Clear(ClearType::CurrentLine),
        SetAttribute(Attribute::Bold),
    )?;
    queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
    queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
    print_truncated(writer, label, &mut remaining)?;
    if show_menu_bar {
        write_menu_bar(writer, menu, frame, &mut remaining)?;
    }
    queue!(writer, SetAttribute(Attribute::Reset))?;
    queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
    queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
    print_truncated(writer, &path, &mut remaining)?;
    if document.buffer.is_dirty() {
        queue_set_foreground_color(writer, &frame, frame.theme.dirty_fg)?;
        queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
        queue!(writer, SetAttribute(Attribute::Bold))?;
        print_truncated(writer, state, &mut remaining)?;
        queue!(writer, SetAttribute(Attribute::Reset))?;
    } else {
        queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
        queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
        print_truncated(writer, state, &mut remaining)?;
    }
    queue!(writer, ResetColor)
}

pub(crate) fn show_menu_bar(frame: RenderFrame) -> bool {
    text_display_width(" kfnotepad ")
        + text_display_width(menu_bar_text())
        + text_display_width(" modified ")
        <= frame.terminal_width
}
