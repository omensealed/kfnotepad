pub(crate) fn no_color_enabled() -> bool {
    no_color_enabled_from(env::var_os("NO_COLOR"))
}

pub(crate) fn no_color_enabled_from(value: Option<std::ffi::OsString>) -> bool {
    matches!(value, Some(value) if !value.is_empty())
}
