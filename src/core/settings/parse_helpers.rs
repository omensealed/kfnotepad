//! Shared scalar parsing for the project configuration formats.

pub(super) fn parse_config_string(value: &str) -> Option<&str> {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| (!value.contains(char::is_whitespace)).then_some(value))
}

pub(super) fn parse_config_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}
