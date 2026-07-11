include!("widget/tree_size.rs");
include!("widget/layout_operate.rs");
include!("widget/update.rs");
include!("widget/draw_overlay.rs");

impl Widget<Message, Theme, iced::Renderer> for GuiInputMethodArea<'_> {
    gui_input_method_tree_size_methods!();
    gui_input_method_layout_operate_methods!();
    gui_input_method_update_methods!();
    gui_input_method_draw_overlay_methods!();
}
