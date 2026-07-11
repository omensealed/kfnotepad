fn platform_config_dir() -> Option<PathBuf> {
    dirs::config_dir().filter(|path| !path.as_os_str().is_empty())
}

fn platform_data_dir() -> Option<PathBuf> {
    dirs::data_dir().filter(|path| !path.as_os_str().is_empty())
}

fn platform_home_dir() -> Option<PathBuf> {
    dirs::home_dir().filter(|path| !path.as_os_str().is_empty())
}
