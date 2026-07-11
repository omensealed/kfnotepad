#[allow(clippy::too_many_arguments)]
pub(crate) fn write_workspace_manager_overlay(
    writer: &mut impl Write,
    manager: &WorkspaceManagerState,
    visible_rows: usize,
    sidebar_width: usize,
    terminal_width: usize,
    body_top: u16,
    theme: EditorTheme,
    no_color: bool,
) -> io::Result<()> {
    let origin_column = sidebar_width as u16;
    let main_width = terminal_width.saturating_sub(sidebar_width).max(1);
    let width = if main_width < 24 {
        main_width
    } else {
        main_width.clamp(24, 64)
    };
    let x = ((main_width.saturating_sub(width)) / 2) as u16;
    let max_rows = visible_rows.saturating_sub(2).max(4);
    let entry_rows = manager.entries.len().min(max_rows.saturating_sub(3)).max(1);
    let height = entry_rows + 3;
    let y = body_top.saturating_add(1);
    let inner_width = width.saturating_sub(2);

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
            queue!(writer, SetAttribute(Attribute::Bold))?;
            write!(writer, "+")?;
            print_truncated(writer, " Workspaces ", &mut remaining)?;
            if remaining > 0 {
                write!(writer, "{}", "-".repeat(remaining))?;
            }
            write!(writer, "+")?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
        } else if row == height - 1 {
            write!(writer, "+")?;
            if inner_width > 0 {
                write!(writer, "{}", "-".repeat(inner_width))?;
            }
            write!(writer, "+")?;
        } else if row == height - 2 {
            write!(writer, "|")?;
            print_truncated(
                writer,
                " Enter open | S save over | D delete | N new | Esc ",
                &mut remaining,
            )?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        } else if manager.entries.is_empty() {
            write!(writer, "|")?;
            print_truncated(writer, " No saved workspaces ", &mut remaining)?;
            if remaining > 0 {
                write!(writer, "{}", " ".repeat(remaining))?;
            }
            write!(writer, "|")?;
        } else {
            let index = manager.scroll + row - 1;
            write!(writer, "|")?;
            if let Some(entry) = manager.entries.get(index) {
                if index == manager.selected {
                    queue_set_foreground_color(writer, no_color, theme.search_fg)?;
                    queue_set_background_color(writer, no_color, theme.search_bg)?;
                    queue!(writer, SetAttribute(Attribute::Bold))?;
                }
                let marker = if index == manager.selected { ">" } else { " " };
                let files = if entry.files == 1 { "file" } else { "files" };
                print_truncated(
                    writer,
                    &format!(" {marker} {}  {} {files} ", entry.name, entry.files),
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

    queue!(writer, SetAttribute(Attribute::Reset), ResetColor)?;
    writer.flush()
}
