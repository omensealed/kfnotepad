//! Menu-group navigation.

use super::*;

pub(crate) fn select_previous_menu_group(runtime: &mut EditorRuntime) {
    if let Some(menu) = &mut runtime.menu {
        menu.group = menu.group.previous();
        menu.selected = 0;
        runtime.status = format!("Menu: {}", menu.group.label());
    }
}

pub(crate) fn select_next_menu_group(runtime: &mut EditorRuntime) {
    if let Some(menu) = &mut runtime.menu {
        menu.group = menu.group.next();
        menu.selected = 0;
        runtime.status = format!("Menu: {}", menu.group.label());
    }
}
