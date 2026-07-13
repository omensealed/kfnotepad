use super::*;

const INSERT_TOKENS: [&str; 5] = ["a", "q", "\u{754c}", "\u{1f642}", "e\u{301}"];
const REPLACEMENT_CHARS: [char; 4] = ['x', 'z', '\u{754c}', '\u{1f642}'];

#[derive(Debug)]
struct LineModel {
    lines: Vec<Vec<String>>,
}

impl LineModel {
    fn sample() -> Self {
        Self {
            lines: vec![vec!["a", "e\u{301}", "\u{1f642}"], vec!["\u{754c}", "z"]]
                .into_iter()
                .map(|line| line.into_iter().map(str::to_owned).collect())
                .collect(),
        }
    }

    fn text(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.concat())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn cursor(&self, row: usize, boundary: usize) -> Cursor {
        Cursor {
            row,
            column: self.lines[row][..boundary]
                .iter()
                .map(|token| token.chars().count())
                .sum(),
        }
    }

    fn nonempty_tokens(&self) -> Vec<(usize, usize)> {
        self.lines
            .iter()
            .enumerate()
            .flat_map(|(row, line)| (0..line.len()).map(move |token| (row, token)))
            .collect()
    }

    fn backspace_boundaries(&self) -> Vec<(usize, usize)> {
        self.lines
            .iter()
            .enumerate()
            .flat_map(|(row, line)| {
                (0..=line.len()).filter_map(move |boundary| {
                    (boundary > 0 || row > 0).then_some((row, boundary))
                })
            })
            .collect()
    }
}

#[derive(Debug)]
struct DeterministicRng(u64);

impl DeterministicRng {
    fn index(&mut self, upper: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 32) as usize) % upper
    }
}

fn apply_generated_edit(
    buffer: &mut TextBuffer,
    model: &mut LineModel,
    rng: &mut DeterministicRng,
) -> &'static str {
    match rng.index(7) {
        0 => {
            let row = rng.index(model.lines.len());
            let boundary = rng.index(model.lines[row].len() + 1);
            let token = INSERT_TOKENS[rng.index(INSERT_TOKENS.len())];
            let cursor = model.cursor(row, boundary);
            buffer
                .insert_text(cursor, token)
                .expect("generated token insertion");
            model.lines[row].insert(boundary, token.to_owned());
            "insert token"
        }
        1 => {
            let row = rng.index(model.lines.len());
            let boundary = rng.index(model.lines[row].len() + 1);
            let cursor = model.cursor(row, boundary);
            buffer
                .insert_newline(cursor.row, cursor.column)
                .expect("generated newline insertion");
            let suffix = model.lines[row].split_off(boundary);
            model.lines.insert(row + 1, suffix);
            "insert newline"
        }
        2 if !model.nonempty_tokens().is_empty() => {
            let positions = model.nonempty_tokens();
            let (row, token) = positions[rng.index(positions.len())];
            let cursor = model.cursor(row, token);
            let replacement = REPLACEMENT_CHARS[rng.index(REPLACEMENT_CHARS.len())];
            buffer
                .replace_char(cursor.row, cursor.column, replacement)
                .expect("generated character replacement");
            model.lines[row][token] = replacement.to_string();
            "replace grapheme"
        }
        3 if !model.nonempty_tokens().is_empty() => {
            let positions = model.nonempty_tokens();
            let (row, token) = positions[rng.index(positions.len())];
            let start = model.cursor(row, token);
            let end = model.cursor(row, token + 1);
            buffer
                .delete_range(start, end)
                .expect("generated token deletion");
            model.lines[row].remove(token);
            "delete grapheme"
        }
        4 if model.lines.len() > 1 => {
            let row = rng.index(model.lines.len() - 1);
            let end = model.cursor(row, model.lines[row].len());
            buffer
                .delete_char(end.row, end.column)
                .expect("generated line join");
            let suffix = model.lines.remove(row + 1);
            model.lines[row].extend(suffix);
            "join lines"
        }
        5 => {
            let row = rng.index(model.lines.len());
            let boundary = rng.index(model.lines[row].len() + 1);
            let cursor = model.cursor(row, boundary);
            let next = buffer
                .insert_text(cursor, "q\n\u{754c}")
                .expect("generated multiline insertion");
            let suffix = model.lines[row].split_off(boundary);
            model.lines[row].push("q".to_owned());
            let mut next_line = vec!["\u{754c}".to_owned()];
            next_line.extend(suffix);
            model.lines.insert(row + 1, next_line);
            assert_eq!(next, model.cursor(row + 1, 1));
            "insert multiline text"
        }
        6 if !model.backspace_boundaries().is_empty() => {
            let positions = model.backspace_boundaries();
            let (row, boundary) = positions[rng.index(positions.len())];
            let cursor = model.cursor(row, boundary);
            buffer
                .delete_before_cursor(cursor)
                .expect("generated backspace");
            if boundary > 0 {
                model.lines[row].remove(boundary - 1);
            } else {
                let suffix = model.lines.remove(row);
                model.lines[row - 1].extend(suffix);
            }
            "backspace"
        }
        _ => {
            buffer
                .insert_char(0, 0, 'a')
                .expect("generated fallback insertion");
            model.lines[0].insert(0, "a".to_owned());
            "fallback insertion"
        }
    }
}

#[test]
fn mixed_delta_history_matches_model_through_complete_undo_and_redo() {
    for seed in [1, 7, 19, 41, 97, 211] {
        let mut rng = DeterministicRng(seed);
        let mut model = LineModel::sample();
        let mut buffer = TextBuffer::from_text(&model.text());
        let mut states = vec![model.text()];

        for step in 0..180 {
            let operation = apply_generated_edit(&mut buffer, &mut model, &mut rng);
            buffer.break_undo_group();
            let expected = model.text();
            assert_eq!(
                buffer.to_text(),
                expected,
                "forward mismatch for seed {seed}, step {step}, operation {operation}"
            );
            states.push(expected);
        }

        assert_eq!(buffer.undo_history.len(), states.len() - 1);
        for (step, expected) in states[..states.len() - 1].iter().enumerate().rev() {
            assert!(
                buffer.undo_last_edit(),
                "missing undo for seed {seed}, step {step}"
            );
            assert_eq!(
                buffer.to_text(),
                *expected,
                "undo mismatch for seed {seed}, step {step}"
            );
        }
        assert!(!buffer.undo_last_edit());

        for (step, expected) in states.iter().enumerate().skip(1) {
            assert!(
                buffer.redo_last_undo(),
                "missing redo for seed {seed}, step {step}"
            );
            assert_eq!(
                buffer.to_text(),
                *expected,
                "redo mismatch for seed {seed}, step {step}"
            );
        }
        assert!(!buffer.redo_last_undo());
    }
}
