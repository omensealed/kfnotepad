//! Advanced Iced widget implementation that forwards IME requests.

use super::*;

#[path = "widget/draw_overlay.rs"]
mod draw_overlay;
#[path = "widget/layout_operate.rs"]
mod layout_operate;
#[path = "widget/tree_size.rs"]
mod tree_size;
#[path = "widget/update.rs"]
mod update;

use draw_overlay::gui_input_method_draw_overlay_methods;
use layout_operate::gui_input_method_layout_operate_methods;
use tree_size::gui_input_method_tree_size_methods;
use update::gui_input_method_update_methods;

impl Widget<Message, Theme, iced::Renderer> for GuiInputMethodArea<'_> {
    gui_input_method_tree_size_methods!();
    gui_input_method_layout_operate_methods!();
    gui_input_method_update_methods!();
    gui_input_method_draw_overlay_methods!();
}
