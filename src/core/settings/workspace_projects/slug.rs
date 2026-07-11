fn gui_workspace_project_slug(name: &str) -> Option<String> {
    let mut slug = String::new();
    let mut previous_dash = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            previous_dash = false;
        } else if character.is_whitespace() || matches!(character, '-' | '_') {
            if !slug.is_empty() && !previous_dash {
                slug.push('-');
                previous_dash = true;
            }
        } else {
            return None;
        }
    }
    if previous_dash {
        slug.pop();
    }
    (!slug.is_empty()).then_some(slug)
}
