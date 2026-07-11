pub(super) fn gui_file_tree_text_size(settings: EditorSettings) -> u32 {
    gui_ui_text_size(settings)
}

pub(super) fn gui_file_tree_icon_size(settings: EditorSettings) -> u32 {
    gui_ui_icon_text_size(settings)
}

pub(super) fn gui_file_tree_view<'a>(
    root: &Path,
    expanded_paths: &HashSet<PathBuf>,
    selected_path: Option<&Path>,
    settings: EditorSettings,
) -> Element<'a, Message> {
    let palette = gui_theme_palette(settings.theme_id);
    let rows = gui_file_tree_rows(root, expanded_paths, selected_path)
        .into_iter()
        .map(|row| gui_file_tree_row(row, settings, palette))
        .collect::<Vec<_>>();

    scrollable(
        column(rows)
            .spacing(GUI_FILE_TREE_ROW_SPACING)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn gui_file_tree_row<'a>(
    row_model: GuiFileTreeRowModel,
    settings: EditorSettings,
    palette: iced::theme::Palette,
) -> Element<'a, Message> {
    let is_dir = row_model.kind != FileSidebarEntryKind::File;
    let path = row_model.path.clone();
    let caret: Element<'a, Message> = if is_dir {
        let glyph = if row_model.expanded {
            nf::oct::OCT_CHEVRON_DOWN
        } else {
            nf::oct::OCT_CHEVRON_RIGHT
        };
        button(
            text(glyph)
                .font(gui_icon_font())
                .size(gui_file_tree_icon_size(settings)),
        )
        .padding(0)
        .width(Length::Fixed(18.0))
        .height(Length::Fixed(22.0))
        .style(move |_theme: &Theme, status| gui_file_tree_button_style(palette, false, status))
        .on_press(Message::BrowserLocalTreeToggle(path.clone()))
        .into()
    } else {
        container(text(""))
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(22.0))
            .into()
    };

    let icon = match row_model.kind {
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory if row_model.expanded => {
            nf::cod::COD_FOLDER_OPENED
        }
        FileSidebarEntryKind::Parent | FileSidebarEntryKind::Directory => nf::cod::COD_FOLDER,
        FileSidebarEntryKind::File => nf::cod::COD_FILE,
    };
    let label_color = gui_file_tree_row_text_color(palette, row_model.selected, row_model.error);
    let content = row![
        text(icon)
            .font(gui_icon_font())
            .size(gui_file_tree_icon_size(settings))
            .color(label_color),
        text(row_model.label)
            .size(gui_file_tree_text_size(settings))
            .color(label_color)
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    let select_path = row_model.path.clone();
    let activate_path = row_model.path.clone();
    let row_content = container(content)
        .padding([1, 3])
        .width(Length::Fill)
        .style(move |_theme| gui_file_tree_row_style(palette, row_model.selected));
    let select_button: Element<'a, Message> = if row_model.error {
        row_content.into()
    } else {
        mouse_area(row_content)
            .on_press(Message::BrowserLocalTreeSelected(select_path, is_dir))
            .on_double_click(Message::BrowserLocalTreeActivated(activate_path, is_dir))
            .interaction(mouse::Interaction::Pointer)
            .into()
    };

    row![
        container(text("")).width(Length::Fixed(row_model.depth as f32 * GUI_FILE_TREE_INDENT)),
        caret,
        select_button,
    ]
    .spacing(1)
    .align_y(Alignment::Center)
    .width(Length::Fill)
    .into()
}
