//! Filtered command-palette overlay and input cursor rendering.

use std::io::{self, Write};

use crossterm::cursor::{MoveTo, Show};
use crossterm::queue;
use crossterm::style::{Attribute, ResetColor, SetAttribute};

use super::colors::{queue_set_background_color, queue_set_foreground_color};
use crate::tui::app::{fit_text_end, print_truncated, text_display_width};
use crate::tui::input::command_palette_candidates;
use crate::tui::menu::CommandPaletteState;
use crate::tui::theme::EditorTheme;

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_command_palette_overlay(
    writer: &mut impl Write,
    palette: &CommandPaletteState,
    visible_rows: usize,
    sidebar_width: usize,
    terminal_width: usize,
    body_top: u16,
    theme: EditorTheme,
    no_color: bool,
) -> io::Result<()> {
    let origin_column = sidebar_width as u16;
    let main_width = terminal_width.saturating_sub(sidebar_width).max(1);
    let width = if main_width < 28 {
        main_width
    } else {
        main_width.clamp(28, 72)
    };
    let x = ((main_width.saturating_sub(width)) / 2) as u16;
    let max_rows = visible_rows.saturating_sub(2).max(5);
    let candidates = command_palette_candidates(&palette.query);
    let entry_rows = candidates.len().min(max_rows.saturating_sub(3)).max(1);
    let height = entry_rows + 3;
    let y = body_top.saturating_add(1);
    let inner_width = width.saturating_sub(2);
    let query_width = inner_width.saturating_sub(text_display_width(" Command: "));
    let query_display = fit_text_end(&palette.query, query_width.max(1));

    for row in 0..height {
        let mut remaining = inner_width;
        queue!(
            writer,
            MoveTo(
                origin_column.saturating_add(x),
                y.saturating_add(row as u16)
            ),
        )?;
        queue_set_foreground_color(writer, no_color, theme.status_fg)?;
        queue_set_background_color(writer, no_color, theme.status_bg)?;
        if row == 0 {
            write!(writer, "+")?;
            queue!(writer, SetAttribute(Attribute::Bold))?;
            print_truncated(
                writer,
                &format!(" Command: {query_display}"),
                &mut remaining,
            )?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "+")?;
        } else if row == height - 1 {
            write!(writer, "+")?;
            if inner_width > 0 {
                write!(writer, "{}", "-".repeat(inner_width))?;
            }
            write!(writer, "+")?;
        } else if candidates.is_empty() {
            write!(writer, "|")?;
            print_truncated(writer, " No matching commands ", &mut remaining)?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        } else {
            let index = palette.scroll + row - 1;
            write!(writer, "|")?;
            if let Some(entry) = candidates.get(index) {
                if index == palette.selected {
                    queue_set_foreground_color(writer, no_color, theme.search_fg)?;
                    queue_set_background_color(writer, no_color, theme.search_bg)?;
                    queue!(writer, SetAttribute(Attribute::Bold))?;
                }
                let marker = if index == palette.selected { ">" } else { " " };
                let shortcut = entry.shortcut.unwrap_or("");
                let suffix = if shortcut.is_empty() {
                    String::new()
                } else {
                    format!("  {shortcut}")
                };
                print_truncated(
                    writer,
                    &format!(" {marker} {}: {}{suffix}", entry.group.label(), entry.label),
                    &mut remaining,
                )?;
                queue!(writer, SetAttribute(Attribute::Reset))?;
                queue_set_foreground_color(writer, no_color, theme.status_fg)?;
                queue_set_background_color(writer, no_color, theme.status_bg)?;
            }
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        }
    }

    let cursor_offset =
        text_display_width("+ Command: ").saturating_add(text_display_width(&query_display));
    queue!(
        writer,
        SetAttribute(Attribute::Reset),
        ResetColor,
        MoveTo(
            origin_column
                .saturating_add(x)
                .saturating_add(cursor_offset.min(width.saturating_sub(1)) as u16),
            y
        ),
        Show
    )?;
    writer.flush()
}
