use super::*;

#[test]
fn gui_tile_window_chrome_uses_compact_gapped_layout() {
    let palette = gui_theme_palette(EditorThemeId::Terminal);
    let active_body = gui_tile_body_style(palette, true);
    let inactive_body = gui_tile_body_style(palette, false);
    let active_title = gui_tile_title_style(palette, true);
    let grid = gui_pane_grid_style(palette);

    assert_eq!(GUI_PANE_GRID_SPACING, 5.0);
    assert_eq!(GUI_EDITOR_PADDING, 2);
    assert_eq!(GUI_TILE_BODY_PADDING, 2);
    assert_eq!(GUI_TILE_TITLE_PADDING, 3);
    assert_eq!(GUI_TILE_CONTROL_SPACING, 1);
    assert_eq!(GUI_PANEL_CONTROL_SPACING, 5);
    assert_eq!(GUI_PANEL_SECTION_SPACING, 6);
    assert_eq!(GUI_PANEL_TREE_TOP_PADDING, 4.0);
    assert_eq!(GUI_CHROME_PADDING, [3, 5]);
    assert_eq!(GUI_LINE_NUMBER_GUTTER_VISIBLE_LINES, 32);
    assert_eq!(GUI_LINE_NUMBER_SEPARATOR_WIDTH, 1.0);
    assert_eq!(GUI_EDITOR_SCROLLBAR_WIDTH, 6.0);
    assert_eq!(active_body.border.radius.top_left, GUI_TILE_RADIUS);
    assert_eq!(active_body.border.width, 1.0);
    assert_eq!(inactive_body.border.width, 1.0);
    assert_eq!(active_body.border.color, palette.primary);
    assert_eq!(
        inactive_body.border.color,
        Color {
            a: 0.55,
            ..palette.primary
        }
    );
    assert_eq!(active_title.text_color, Some(palette.primary));
    assert_eq!(grid.hovered_region.border.width, 0.0);
    assert_eq!(grid.hovered_region.border.color, Color::TRANSPARENT);
    assert!(matches!(
        grid.hovered_region.background,
        Background::Color(Color::TRANSPARENT)
    ));
    assert_eq!(grid.hovered_split.width, 1.0);
    assert_eq!(grid.picked_split.color, palette.primary);
}

#[test]
fn gui_editor_scrollbar_model_is_thin_and_proportional() {
    let model = gui_editor_scrollbar_model(100, 41, 20, 200.0);

    assert!(model.visible);
    assert_eq!(GUI_EDITOR_SCROLLBAR_WIDTH, 6.0);
    assert_eq!(model.track_height, 200.0);
    assert_eq!(model.thumb_height, 40.0);
    assert_eq!(model.thumb_top, 80.0);
    assert_eq!(model.page_delta, 20);

    let hidden = gui_editor_scrollbar_model(5, 1, 20, 200.0);
    assert!(!hidden.visible);
    assert_eq!(hidden.thumb_height, 200.0);
}

#[test]
fn gui_equalized_tile_layout_places_remainder_on_right() {
    let three = equalized_tile_layout_node(3).expect("three layout");
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = three
    else {
        panic!("expected three-tile split");
    };
    assert_eq!(axis, GuiLayoutAxis::Vertical);
    assert_eq!(ratio_per_mille, 500);
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 2 });
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = *first
    else {
        panic!("expected first column row split");
    };
    assert_eq!(axis, GuiLayoutAxis::Horizontal);
    assert_eq!(ratio_per_mille, 500);
    assert_eq!(*first, GuiLayoutNode::Leaf { ordinal: 0 });
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 1 });

    let five = equalized_tile_layout_node(5).expect("five layout");
    let GuiLayoutNode::Split {
        axis,
        ratio_per_mille,
        first,
        second,
    } = five
    else {
        panic!("expected five-tile split");
    };
    assert_eq!(axis, GuiLayoutAxis::Vertical);
    assert_eq!(ratio_per_mille, 666);
    assert_eq!(*second, GuiLayoutNode::Leaf { ordinal: 4 });
    assert_eq!(layout_leaf_ordinals(&first), vec![0, 1, 2, 3]);
}
