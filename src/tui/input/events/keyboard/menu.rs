//! Document menu keyboard handling.

use super::*;

pub(crate) fn handle_menu_key_event(
    document: &mut TextDocument,
    cursor: &mut Cursor,
    runtime: &mut EditorRuntime,
    event: KeyEvent,
) -> bool {
    match event.code {
        KeyCode::Esc | KeyCode::F(10) => {
            runtime.menu = None;
            runtime.status = String::from("Menu closed");
        }
        KeyCode::BackTab => {
            select_previous_menu_group(runtime);
        }
        KeyCode::Tab => {
            if event.modifiers.contains(KeyModifiers::SHIFT) {
                select_previous_menu_group(runtime);
            } else {
                select_next_menu_group(runtime);
            }
        }
        KeyCode::Left => {
            select_previous_menu_group(runtime);
        }
        KeyCode::Right => {
            select_next_menu_group(runtime);
        }
        KeyCode::Home => {
            if let Some(menu) = &mut runtime.menu {
                menu.selected = 0;
            }
        }
        KeyCode::End => {
            if let Some(menu) = &mut runtime.menu {
                menu.selected = menu.group.items().len().saturating_sub(1);
            }
        }
        KeyCode::Up => {
            if let Some(menu) = &mut runtime.menu {
                let item_count = menu.group.items().len();
                menu.selected = if menu.selected == 0 {
                    item_count.saturating_sub(1)
                } else {
                    menu.selected.saturating_sub(1)
                };
            }
        }
        KeyCode::Down => {
            if let Some(menu) = &mut runtime.menu {
                let item_count = menu.group.items().len().max(1);
                menu.selected = (menu.selected + 1) % item_count;
            }
        }
        KeyCode::Enter => {
            let command = runtime.menu.and_then(|menu| {
                menu.group
                    .items()
                    .get(menu.selected)
                    .map(|item| item.command)
            });
            runtime.menu = None;
            if let Some(command) = command {
                return run_menu_command(command, document, cursor, runtime);
            }
        }
        _ => {}
    }
    false
}
