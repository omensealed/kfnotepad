//! Environment overrides and platform-standard base directories.

use std::path::PathBuf;

fn environment_dir(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn platform_config_dir() -> Option<PathBuf> {
    dirs::config_dir().filter(|path| !path.as_os_str().is_empty())
}

fn platform_data_dir() -> Option<PathBuf> {
    dirs::data_dir().filter(|path| !path.as_os_str().is_empty())
}

pub(super) fn platform_home_dir() -> Option<PathBuf> {
    dirs::home_dir().filter(|path| !path.as_os_str().is_empty())
}

pub(super) fn current_config_base_dir() -> Option<PathBuf> {
    environment_dir("XDG_CONFIG_HOME").or_else(platform_config_dir)
}

pub(super) fn current_data_base_dir() -> Option<PathBuf> {
    environment_dir("XDG_DATA_HOME").or_else(platform_data_dir)
}
