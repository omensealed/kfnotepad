use super::*;

#[test]
fn status_line_preserves_cursor_and_mode_metadata() {
    let status = compose_status_line(
        " very long transient status text that can be shortened ",
        " Ln 12, Col 80 | num:on | wrap:off | x:42 | nocturne | modified ",
        64,
    );

    assert_eq!(status.chars().count(), 64);
    assert!(status.contains("Col 80"));
    assert!(status.contains("wrap:off"));
    assert!(status.contains("x:42"));
    assert!(status.contains("modified"));
}

#[test]
fn search_status_preserves_query_tail_and_cursor() {
    let status = compose_prompt_status_line(
        "Search: ",
        "a very long search query",
        " Ln 1, Col 1 | num:on | wrap:off | x:0 | nocturne | saved ",
        72,
    );

    assert_eq!(status.text.chars().count(), 72);
    assert!(status.text.contains("Search:"));
    assert!(status.text.contains("…"));
    assert!(status.text.contains("ry"));
    assert!(status.text.contains("Col 1"));
    assert!(status.cursor_column.is_some());
}
