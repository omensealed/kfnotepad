//! Conditional Crossterm color queue helpers for `NO_COLOR` rendering.

use std::io::{self, Write};

use crossterm::queue;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};

pub(super) fn queue_set_foreground_color(
    writer: &mut impl Write,
    no_color: bool,
    color: Color,
) -> io::Result<()> {
    if no_color {
        return Ok(());
    }
    queue!(writer, SetForegroundColor(color))
}

pub(super) fn queue_set_background_color(
    writer: &mut impl Write,
    no_color: bool,
    color: Color,
) -> io::Result<()> {
    if no_color {
        return Ok(());
    }
    queue!(writer, SetBackgroundColor(color))
}
