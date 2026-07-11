use std::env;

use crossterm::event::KeyboardEnhancementFlags;

pub(crate) fn editor_keyboard_enhancement_flags() -> KeyboardEnhancementFlags {
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
}

pub(crate) fn supports_tui_terminal() -> bool {
    let term = env::var_os("TERM")
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    !matches!(term.as_str(), "dumb" | "unknown" | "unknownterm")
}
