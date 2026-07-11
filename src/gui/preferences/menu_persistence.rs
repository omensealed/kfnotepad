impl KfnotepadGui {
    pub(super) fn run_menu_command(&mut self, command: GuiMenuCommand) -> Task<Message> {
        match command {
            GuiMenuCommand::NewTile => self.create_new_tile(),
            GuiMenuCommand::Open => return self.request_open_dialog(),
            GuiMenuCommand::OpenPath => self.show_path_prompt(GuiPathPrompt::Open),
            GuiMenuCommand::Save => return self.request_save_active_tile_async(),
            GuiMenuCommand::SaveAs => return self.request_save_as_dialog(),
            GuiMenuCommand::SaveAsPath => self.show_path_prompt(GuiPathPrompt::SaveAs),
            GuiMenuCommand::ClosePane => self.close_active_pane(),
            GuiMenuCommand::Quit => return window::latest().map(Message::QuitLatestWindow),
            GuiMenuCommand::OpenManagedNote => self.show_path_prompt(GuiPathPrompt::ManagedNote),
            GuiMenuCommand::ListManagedNotes => self.list_managed_notes_panel(),
            GuiMenuCommand::Copy => return self.copy_active_selection(),
            GuiMenuCommand::Cut => return self.cut_active_selection(),
            GuiMenuCommand::Paste => {
                self.status_message = "paste requested".to_string();
                return clipboard::read().map(Message::ClipboardPasted);
            }
            GuiMenuCommand::Undo => self.undo_active_edit(),
            GuiMenuCommand::Redo => self.redo_active_edit(),
            GuiMenuCommand::SelectAll => self.select_all_active_editor(),
            GuiMenuCommand::FindNext => self.search_active(false),
            GuiMenuCommand::FindPrevious => self.search_active(true),
            GuiMenuCommand::ToggleBrowser => self.toggle_left_panel(),
            GuiMenuCommand::CycleTheme => self.cycle_theme(),
            GuiMenuCommand::CycleSyntaxTheme => self.cycle_syntax_theme(),
            GuiMenuCommand::ToggleReaderMode => self.toggle_reader_mode(),
            GuiMenuCommand::GoDocumentStart => self.go_active_document_start(),
            GuiMenuCommand::GoDocumentEnd => self.go_active_document_end(),
            GuiMenuCommand::GoToLine => self.go_active_line(),
            GuiMenuCommand::ToggleMinimize => self.toggle_active_minimize(),
            GuiMenuCommand::ToggleMaximize => self.toggle_active_maximize(),
            GuiMenuCommand::EqualizeTiles => self.equalize_tile_layout(),
            GuiMenuCommand::MoveLeft => self.move_active_pane(pane_grid::Direction::Left),
            GuiMenuCommand::MoveRight => self.move_active_pane(pane_grid::Direction::Right),
            GuiMenuCommand::MoveUp => self.move_active_pane(pane_grid::Direction::Up),
            GuiMenuCommand::MoveDown => self.move_active_pane(pane_grid::Direction::Down),
            GuiMenuCommand::OpenHelp => self.open_help_document(),
        }
        if matches!(
            command,
            GuiMenuCommand::NewTile
                | GuiMenuCommand::ClosePane
                | GuiMenuCommand::ToggleMinimize
                | GuiMenuCommand::ToggleMaximize
                | GuiMenuCommand::EqualizeTiles
                | GuiMenuCommand::MoveLeft
                | GuiMenuCommand::MoveRight
                | GuiMenuCommand::MoveUp
                | GuiMenuCommand::MoveDown
        ) {
            self.persist_last_workspace_if_enabled();
        }
        Task::none()
    }

    pub(super) fn persist_settings(&mut self) {
        let Some(config_path) = self.config_path.as_deref() else {
            return;
        };
        if let Err(error) = save_editor_settings(config_path, self.settings) {
            self.status_message = format!("settings save failed: {error}");
        }
    }

    pub(super) fn persist_layout(&mut self) {
        let Some(layout_path) = self.layout_path.as_deref() else {
            return;
        };
        let Some(layout) = gui_layout_from_state(
            &self.panes,
            &self.workspace,
            self.browser_visible,
            self.browser_width,
        ) else {
            return;
        };
        if let Err(error) = save_gui_layout(layout_path, &layout) {
            self.status_message = format!("layout save failed: {error}");
        }
    }
}
