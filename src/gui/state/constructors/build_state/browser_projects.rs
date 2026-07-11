#[cfg(test)]
fn build_gui_browser(
    current_dir: &std::path::Path,
    status_messages: &mut Vec<String>,
) -> GuiBrowserBuild {
    match GuiFileBrowser::load(current_dir.to_path_buf()) {
        Ok(browser) => {
            let root = browser.sidebar.current_dir.clone();
            let mut expanded = HashSet::new();
            expanded.insert(root.clone());
            let browser_tree_rows = gui_file_tree_rows(&root, &expanded, Some(&root));
            GuiBrowserBuild {
                browser: Some(browser),
                browser_tree_rows,
                browser_expanded_paths: expanded,
            }
        }
        Err(error) => {
            status_messages.push(format!(
                "file browser unavailable for {}: {error}",
                current_dir.display()
            ));
            GuiBrowserBuild {
                browser: None,
                browser_tree_rows: Vec::new(),
                browser_expanded_paths: HashSet::new(),
            }
        }
    }
}

#[cfg(not(test))]
fn build_gui_browser(
    _current_dir: &std::path::Path,
    _status_messages: &mut Vec<String>,
) -> GuiBrowserBuild {
    GuiBrowserBuild {
        browser: None,
        browser_tree_rows: Vec::new(),
        browser_expanded_paths: HashSet::new(),
    }
}

fn load_gui_workspace_project_entries(
    workspace_projects_dir: Option<&std::path::Path>,
    status_messages: &mut Vec<String>,
) -> Vec<GuiWorkspaceProjectEntry> {
    workspace_projects_dir
        .and_then(|path| match list_gui_workspace_projects(path) {
            Ok(projects) => Some(projects),
            Err(error) => {
                status_messages.push(format!("workspace projects unavailable: {error}"));
                None
            }
        })
        .unwrap_or_default()
}
