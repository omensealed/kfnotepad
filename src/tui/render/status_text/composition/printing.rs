//! Display-width-bounded terminal text printing.

use super::*;

pub(crate) fn print_truncated(
    writer: &mut impl Write,
    text: &str,
    remaining_columns: &mut usize,
) -> io::Result<()> {
    if *remaining_columns == 0 {
        return Ok(());
    }

    let mut visible = String::new();
    let mut used_columns = 0;
    for character in text.chars() {
        let width = character_display_width(character, used_columns);
        if width > 0 && used_columns + width > *remaining_columns {
            break;
        }
        visible.push(character);
        used_columns += width;
    }
    *remaining_columns = remaining_columns.saturating_sub(used_columns);
    queue!(writer, Print(visible))
}
