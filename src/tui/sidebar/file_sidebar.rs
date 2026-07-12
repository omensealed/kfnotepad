//! File sidebar rows, selection, and current-directory header rendering.

use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Attribute, ResetColor, SetAttribute};
use kfnotepad::FileSidebarState;

use super::colors::{queue_set_background_color, queue_set_foreground_color};
use crate::tui::app::{fit_text_end, print_truncated, SIDEBAR_WIDTH};
use crate::tui::theme::EditorTheme;

pub(crate) fn render_file_sidebar(
    writer: &mut impl Write,
    sidebar: &FileSidebarState,
    visible_rows: usize,
    theme: EditorTheme,
    no_color: bool,
) -> io::Result<()> {
    let width = SIDEBAR_WIDTH;
    let title = fit_text_end(
        &format!(" Files: {} ", sidebar.current_dir.display()),
        width,
    );
    let mut remaining = width;
    queue!(writer, MoveTo(0, 0), SetAttribute(Attribute::Bold))?;
    queue_set_foreground_color(writer, no_color, theme.status_fg)?;
    queue_set_background_color(writer, no_color, theme.status_bg)?;
    print_truncated(writer, &title, &mut remaining)?;

    for row in 0..visible_rows {
        let entry_index = sidebar.scroll + row;
        let screen_row = (row + 1) as u16;
        let mut remaining = width;
        queue!(writer, MoveTo(0, screen_row),)?;
        if entry_index == sidebar.selected {
            queue!(writer, SetAttribute(Attribute::Bold))?;
            queue_set_foreground_color(writer, no_color, theme.status_fg)?;
            queue_set_background_color(writer, no_color, theme.status_bg)?;
        } else {
            queue!(writer, SetAttribute(Attribute::Reset))?;
            queue_set_foreground_color(writer, no_color, theme.header_fg)?;
            queue_set_background_color(writer, no_color, theme.header_bg)?;
        }
        let label = sidebar
            .entries
            .get(entry_index)
            .map(|entry| entry.label.as_str())
            .unwrap_or("");
        print_truncated(writer, &format!(" {label}"), &mut remaining)?;
    }

    let selected_row = sidebar.selected.saturating_sub(sidebar.scroll) + 1;
    queue!(
        writer,
        SetAttribute(Attribute::Reset),
        ResetColor,
        MoveTo(2, selected_row.min(visible_rows) as u16)
    )?;
    writer.flush()
}
