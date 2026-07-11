impl KfnotepadGui {
    fn new_with_paths(
        launch: GuiLaunch,
        current_dir: PathBuf,
        config_path: Option<PathBuf>,
        layout_path: Option<PathBuf>,
        workspace_project_path: Option<PathBuf>,
        workspace_projects_dir: Option<PathBuf>,
    ) -> Self {
        let mut status_messages = Vec::new();
        let settings = load_gui_settings(config_path.as_deref(), &mut status_messages);
        let launch_documents = load_gui_launch_documents(
            launch,
            &current_dir,
            settings,
            workspace_project_path,
            workspace_projects_dir.as_deref(),
            &mut status_messages,
        );
        let show_startup_help_panel = launch_documents.show_startup_help_panel;
        let (mut workspace, pane_states) =
            build_workspace_and_pane_states(launch_documents.documents);
        let restored_layout = launch_documents.project_layout.clone().or_else(|| {
            layout_path
                .as_deref()
                .and_then(|path| load_gui_layout(path, pane_states.len()))
        });
        let browser_visible = restored_layout
            .as_ref()
            .is_none_or(|layout| layout.browser_visible);
        let browser_width = restored_layout
            .as_ref()
            .and_then(|layout| layout.browser_width_px)
            .map(f32::from)
            .map(clamp_browser_width)
            .unwrap_or(GUI_BROWSER_WIDTH_DEFAULT);
        let GuiPaneBuild {
            panes,
            minimized_panes,
            active_pane,
        } = build_gui_panes(
            &mut workspace,
            pane_states,
            restored_layout,
            launch_documents.project_active_ordinal,
            &mut status_messages,
        );
        let GuiBrowserBuild {
            browser,
            browser_tree,
            browser_expanded_paths,
        } = build_gui_browser(&current_dir, &mut status_messages);
        let left_panel = GuiLeftPanelState {
            visible: browser_visible,
            mode: GuiLeftPanelMode::Files,
        };
        let notes_dir = current_managed_notes_dir().ok();
        let workspace_projects = load_gui_workspace_project_entries(
            workspace_projects_dir.as_deref(),
            &mut status_messages,
        );

        let mut state = Self {
            workspace,
            panes,
            active_pane,
            minimized_panes,
            browser,
            browser_tree,
            browser_expanded_paths,
            browser_selected_path: None,
            browser_visible,
            browser_width,
            left_panel,
            current_dir,
            notes_dir,
            workspace_projects_dir,
            workspace_projects,
            workspace_project_name: String::new(),
            pending_project_delete: None,
            pending_browser_delete: None,
            #[cfg(test)]
            spawned_workspace_project_paths: Vec::new(),
            path_prompt: None,
            path_prompt_value: String::new(),
            notes_panel: None,
            pending_managed_note_delete: None,
            file_snapshots: HashMap::new(),
            external_file_check_in_flight: false,
            external_edit_locks: HashSet::new(),
            syntax_caches: HashMap::new(),
            replacement_pointer_point: None,
            replacement_drag: None,
            replacement_drag_edge: None,
            replacement_scrollbar_drag: None,
            replacement_scrollbar_pointer: None,
            replacement_ime_preedit: None,
            replacement_overwrite_mode: false,
            pending_project_open: None,
            pending_close_tile: None,
            pending_app_quit: false,
            search_query: String::new(),
            search_history: Vec::new(),
            search_history_open: false,
            search_highlight: None,
            reader_scroll_accumulator: 0.0,
            go_to_line_query: String::new(),
            syntax_highlighter: SyntaxHighlighter::default(),
            settings,
            config_path,
            layout_path,
            status_message: status_messages.join(" | "),
            show_startup_help_panel,
        };
        state.refresh_all_file_snapshots();
        state.refresh_visible_syntax_caches();
        state.persist_last_workspace_if_enabled();
        state
    }
}
