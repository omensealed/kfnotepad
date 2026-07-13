use std::sync::LazyLock;

use divan::{
    counter::{BytesCount, ItemsCount},
    Bencher,
};
use kfnotepad::{Cursor, TextBuffer, MAX_TEXT_FILE_BYTES};

const PASTE_BYTES: usize = 100 * 1024;
const TYPED_CHARS: usize = 1_000;
const UNDO_OPERATIONS: usize = 200;

static NEAR_LIMIT_TEXT: LazyLock<String> = LazyLock::new(|| {
    synthetic_text(
        MAX_TEXT_FILE_BYTES as usize - 1024,
        "unique-kfnotepad-benchmark-needle",
    )
});
static ONE_MIB_TEXT: LazyLock<String> =
    LazyLock::new(|| synthetic_text(1024 * 1024, "middle-marker"));
static ONE_MIB_LINE: LazyLock<String> = LazyLock::new(|| "a".repeat(1024 * 1024));
static LARGE_PASTE: LazyLock<String> = LazyLock::new(|| "paste payload ".repeat(8_534));

fn main() {
    divan::main();
}

fn synthetic_text(target_bytes: usize, final_marker: &str) -> String {
    let line = "fn synthetic_line() { let value = 12345; }\n";
    let marker = format!("{final_marker}\n");
    let body_bytes = target_bytes.saturating_sub(marker.len());
    let mut text = String::with_capacity(target_bytes);

    while text.len() + line.len() <= body_bytes {
        text.push_str(line);
    }
    text.extend(std::iter::repeat_n('x', body_bytes - text.len()));
    text.push_str(&marker);
    text
}

#[divan::bench]
fn construct_near_limit(bencher: Bencher<'_, '_>) {
    bencher
        .counter(BytesCount::of_str(NEAR_LIMIT_TEXT.as_str()))
        .bench(|| TextBuffer::from_text(divan::black_box(NEAR_LIMIT_TEXT.as_str())));
}

#[divan::bench]
fn serialize_near_limit(bencher: Bencher<'_, '_>) {
    let buffer = TextBuffer::from_text(NEAR_LIMIT_TEXT.as_str());
    bencher
        .counter(BytesCount::of_str(NEAR_LIMIT_TEXT.as_str()))
        .bench(|| divan::black_box(&buffer).to_text());
}

#[divan::bench]
fn search_near_limit_end(bencher: Bencher<'_, '_>) {
    let buffer = TextBuffer::from_text(NEAR_LIMIT_TEXT.as_str());
    bencher
        .counter(BytesCount::of_str(NEAR_LIMIT_TEXT.as_str()))
        .bench(|| {
            divan::black_box(&buffer).find_next(
                divan::black_box("unique-kfnotepad-benchmark-needle"),
                Cursor { row: 0, column: 0 },
            )
        });
}

#[divan::bench]
fn search_near_limit_missing(bencher: Bencher<'_, '_>) {
    let buffer = TextBuffer::from_text(NEAR_LIMIT_TEXT.as_str());
    bencher
        .counter(BytesCount::of_str(NEAR_LIMIT_TEXT.as_str()))
        .bench(|| {
            divan::black_box(&buffer).find_next(
                divan::black_box("not-present-in-synthetic-document"),
                Cursor { row: 0, column: 0 },
            )
        });
}

#[divan::bench]
fn paste_100_kib(bencher: Bencher<'_, '_>) {
    assert!(LARGE_PASTE.len() >= PASTE_BYTES);
    let paste = &LARGE_PASTE[..PASTE_BYTES];
    bencher
        .counter(BytesCount::of_str(paste))
        .with_inputs(|| TextBuffer::from_text(ONE_MIB_TEXT.as_str()))
        .bench_refs(|buffer| {
            buffer
                .insert_text(Cursor { row: 0, column: 0 }, divan::black_box(paste))
                .expect("synthetic paste remains under the text limit")
        });
}

#[divan::bench]
fn delete_100_kib(bencher: Bencher<'_, '_>) {
    bencher
        .counter(BytesCount::new(PASTE_BYTES))
        .with_inputs(|| TextBuffer::from_text(ONE_MIB_LINE.as_str()))
        .bench_refs(|buffer| {
            buffer
                .delete_range(
                    Cursor { row: 0, column: 0 },
                    Cursor {
                        row: 0,
                        column: PASTE_BYTES,
                    },
                )
                .expect("synthetic delete range remains valid")
        });
}

#[divan::bench]
fn type_1000_ascii_chars(bencher: Bencher<'_, '_>) {
    bencher
        .counter(ItemsCount::new(TYPED_CHARS))
        .with_inputs(|| TextBuffer::from_text(""))
        .bench_refs(|buffer| {
            for column in 0..TYPED_CHARS {
                buffer
                    .insert_char(0, column, 'x')
                    .expect("synthetic typed insert");
            }
        });
}

#[divan::bench]
fn undo_200_operations(bencher: Bencher<'_, '_>) {
    bencher
        .counter(ItemsCount::new(UNDO_OPERATIONS))
        .with_inputs(|| {
            let mut buffer = TextBuffer::from_text("a");
            for index in 0..UNDO_OPERATIONS {
                let value = if index % 2 == 0 { 'b' } else { 'a' };
                buffer
                    .replace_char(0, 0, value)
                    .expect("prepare synthetic undo history");
            }
            buffer
        })
        .bench_refs(|buffer| {
            for _ in 0..UNDO_OPERATIONS {
                assert!(buffer.undo_last_edit());
            }
        });
}
