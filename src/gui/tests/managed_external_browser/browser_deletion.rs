use super::*;

#[test]
fn gui_browser_delete_file_requires_confirmation() {
    let temp = TempArea::new("gui-browser-delete-file");
    let file = temp.path("delete-me.txt");
    fs::write(&file, "delete\n").expect("write delete file");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "delete-me.txt")
        .expect("delete file entry");
    state.select_browser_entry(index);

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(file.exists());
    assert!(state.status_message.contains("click delete again"));

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(!file.exists());
    assert_eq!(
        state.status_message,
        format!("moved file to trash {}", file.display())
    );
}

#[test]
fn gui_browser_delete_targets_tree_selected_nested_file() {
    let temp = TempArea::new("gui-browser-delete-tree-file");
    let subdir = temp.path("subdir");
    fs::create_dir(&subdir).expect("create subdir");
    let file = subdir.join("delete-me.txt");
    fs::write(&file, "delete\n").expect("write delete file");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );

    let _ = update(
        &mut state,
        Message::BrowserLocalTreeSelected(file.clone(), false),
    );
    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(file.exists());
    assert!(state.status_message.contains("click delete again"));

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(!file.exists());
    assert_eq!(
        state.status_message,
        format!("moved file to trash {}", file.display())
    );
}

#[test]
fn gui_browser_delete_directory_warns_and_removes_tree_after_confirmation() {
    let temp = TempArea::new("gui-browser-delete-dir");
    let directory = temp.path("delete-dir");
    fs::create_dir(&directory).expect("create delete dir");
    fs::create_dir(directory.join("child")).expect("create child dir");
    fs::write(directory.join("child").join("nested.txt"), "nested\n").expect("write nested");
    let mut state = KfnotepadGui::new_with_paths(
        GuiLaunch {
            requested_paths: Vec::new(),
        },
        temp.root.clone(),
        None,
        None,
        None,
        None,
    );
    let index = state
        .browser
        .as_ref()
        .expect("browser")
        .sidebar
        .entries
        .iter()
        .position(|entry| entry.label == "delete-dir/")
        .expect("delete dir entry");
    state.select_browser_entry(index);

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(directory.exists());
    assert!(state.status_message.contains("all subdirectories/files"));

    let _ = update(&mut state, Message::BrowserDeleteSelectedRequested);
    assert!(!directory.exists());
    assert_eq!(
        state.status_message,
        format!("moved directory to trash {}", directory.display())
    );
}
