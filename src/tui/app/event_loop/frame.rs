//! Frame preparation, editor rendering, and runtime overlays.

#[path = "frame/layout.rs"]
mod layout;
#[path = "frame/overlays.rs"]
mod overlays;
#[path = "frame/render.rs"]
mod render;

pub(in crate::tui::app::event_loop) use render::render_frame;
