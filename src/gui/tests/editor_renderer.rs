//! Editor rendering, editing, pointer, keyboard, IME, and settings tests.

use super::*;

#[path = "editor_renderer/editing_clipboard.rs"]
mod editing_clipboard;
#[path = "editor_renderer/keyboard_ime_settings.rs"]
mod keyboard_ime_settings;
#[path = "editor_renderer/pointer_scroll.rs"]
mod pointer_scroll;
#[path = "editor_renderer/surface_cache_adapter.rs"]
mod surface_cache_adapter;
#[path = "editor_renderer/viewport_rendering.rs"]
mod viewport_rendering;
