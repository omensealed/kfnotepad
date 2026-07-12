//! Header menu-bar rendering.

use super::*;

pub(super) fn write_menu_bar(
    writer: &mut impl Write,
    menu: Option<MenuState>,
    frame: RenderFrame,
    remaining: &mut usize,
) -> io::Result<()> {
    for group in MENU_GROUPS {
        let active = menu.is_some_and(|menu| menu.group == group);
        if active {
            queue_set_foreground_color(writer, &frame, frame.theme.status_fg)?;
            queue_set_background_color(writer, &frame, frame.theme.status_bg)?;
            queue!(writer, SetAttribute(Attribute::Bold))?;
        } else {
            queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
            queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
            queue!(writer, SetAttribute(Attribute::Reset))?;
        }
        let group_label = format!(" {} ", group.label());
        print_truncated(writer, &group_label, remaining)?;
    }
    queue_set_foreground_color(writer, &frame, frame.theme.header_fg)?;
    queue_set_background_color(writer, &frame, frame.theme.header_bg)?;
    queue!(writer, SetAttribute(Attribute::Reset))?;
    print_truncated(writer, "|", remaining)
}
