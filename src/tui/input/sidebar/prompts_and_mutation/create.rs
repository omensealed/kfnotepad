pub(crate) fn create_sidebar_file(runtime: &mut EditorRuntime) {
    let name = match validated_sidebar_child_name(&runtime.sidebar_query) {
        Ok(name) => name.to_string(),
        Err(error) => {
            runtime.status = error;
            return;
        }
    };
    let Some(parent) = sidebar_target_directory(runtime) else {
        runtime.status = String::from("Files unavailable");
        return;
    };
    let path = parent.join(&name);

    match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(created_file) => {
            #[cfg(not(unix))]
            let _ = &created_file;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = created_file.set_permissions(fs::Permissions::from_mode(0o600));
            }
            refresh_sidebar_after_path_in_dir(runtime, &parent, &path);
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = format!("Created file {name}");
        }
        Err(error) => runtime.status = format!("Create file failed: {error}"),
    }
}

pub(crate) fn create_sidebar_directory(runtime: &mut EditorRuntime) {
    let name = match validated_sidebar_child_name(&runtime.sidebar_query) {
        Ok(name) => name.to_string(),
        Err(error) => {
            runtime.status = error;
            return;
        }
    };
    let Some(parent) = sidebar_target_directory(runtime) else {
        runtime.status = String::from("Files unavailable");
        return;
    };
    let path = parent.join(&name);

    match fs::create_dir(&path) {
        Ok(()) => {
            refresh_sidebar_after_path_in_dir(runtime, &parent, &path);
            runtime.sidebar_prompt = None;
            runtime.sidebar_query.clear();
            runtime.status = format!("Created directory {name}/");
        }
        Err(error) => runtime.status = format!("Create directory failed: {error}"),
    }
}
