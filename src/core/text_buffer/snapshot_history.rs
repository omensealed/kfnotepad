//! Snapshot sizing and byte-budgeted mixed history queue operations.

use super::*;

impl BufferSnapshot {
    pub(super) fn from_buffer(buffer: &TextBuffer) -> Self {
        Self {
            lines: buffer.lines.clone(),
            trailing_newline: buffer.trailing_newline,
            byte_size: buffer_bytes(&buffer.lines, buffer.trailing_newline),
        }
    }
}

pub(super) fn buffer_bytes(lines: &[String], trailing_newline: bool) -> usize {
    let line_bytes: usize = lines.iter().map(String::len).sum();
    let newline_bytes = lines.len().saturating_sub(1);
    let trailing_newline_byte = usize::from(trailing_newline);
    line_bytes + newline_bytes + trailing_newline_byte
}

pub(crate) fn push_history_snapshot(
    history: &mut VecDeque<HistoryEntry>,
    used_bytes: &mut usize,
    snapshot: BufferSnapshot,
    max_entries: usize,
    max_bytes: usize,
) {
    push_history_entry(
        history,
        used_bytes,
        HistoryEntry::Snapshot(snapshot),
        max_entries,
        max_bytes,
    );
}

pub(crate) fn push_history_entry(
    history: &mut VecDeque<HistoryEntry>,
    used_bytes: &mut usize,
    entry: HistoryEntry,
    max_entries: usize,
    max_bytes: usize,
) {
    *used_bytes = used_bytes.saturating_add(entry.byte_size());
    history.push_back(entry);

    while (history.len() > max_entries || *used_bytes > max_bytes) && !history.is_empty() {
        if let Some(removed) = history.pop_front() {
            *used_bytes = used_bytes.saturating_sub(removed.byte_size());
        }
    }
}

pub(crate) fn pop_history_entry(
    history: &mut VecDeque<HistoryEntry>,
    used_bytes: &mut usize,
) -> Option<HistoryEntry> {
    let entry = history.pop_back()?;
    *used_bytes = used_bytes.saturating_sub(entry.byte_size());
    Some(entry)
}
