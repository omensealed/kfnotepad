use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy)]
struct GraphemeColumn {
    start: usize,
    end: usize,
    is_word: bool,
}

fn previous_grapheme_column(line: &str, column: usize) -> Result<usize, BufferError> {
    let columns = line.chars().count();
    if column > columns {
        return Err(BufferError::ColumnOutOfBounds { column, columns });
    }

    let mut previous = 0;
    for (start, end) in grapheme_column_ranges(line) {
        if end >= column {
            return Ok(start);
        }
        previous = start;
    }
    Ok(previous)
}

fn next_grapheme_column(line: &str, column: usize) -> Result<usize, BufferError> {
    let columns = line.chars().count();
    if column > columns {
        return Err(BufferError::ColumnOutOfBounds { column, columns });
    }

    for (start, end) in grapheme_column_ranges(line) {
        if start <= column && column < end {
            return Ok(end);
        }
        if start > column {
            return Ok(start);
        }
    }
    Ok(columns)
}

fn nearest_grapheme_boundary_column(line: &str, column: usize) -> Result<usize, BufferError> {
    let columns = line.chars().count();
    if column > columns {
        return Err(BufferError::ColumnOutOfBounds { column, columns });
    }

    for (start, end) in grapheme_column_ranges(line) {
        if start <= column && column <= end {
            let before = column.saturating_sub(start);
            let after = end.saturating_sub(column);
            return Ok(if before < after { start } else { end });
        }
    }
    Ok(columns)
}

fn grapheme_char_range_at_column(
    line: &str,
    column: usize,
) -> Result<Option<(usize, usize)>, BufferError> {
    let columns = line.chars().count();
    if column > columns {
        return Err(BufferError::ColumnOutOfBounds { column, columns });
    }

    for (start, end) in grapheme_column_ranges(line) {
        if start <= column && column < end {
            return Ok(Some((start, end)));
        }
    }
    Ok(None)
}

fn grapheme_range_boundary_columns_for_line(
    line: &str,
    start_column: usize,
    end_column: usize,
) -> Result<(usize, usize), BufferError> {
    let columns = line.chars().count();
    if start_column > columns {
        return Err(BufferError::ColumnOutOfBounds {
            column: start_column,
            columns,
        });
    }
    if end_column > columns {
        return Err(BufferError::ColumnOutOfBounds {
            column: end_column,
            columns,
        });
    }

    let start = grapheme_range_start_boundary_column(line, start_column);
    let end = grapheme_range_end_boundary_column(line, end_column);
    Ok((start.min(end), end.max(start)))
}

fn grapheme_range_start_boundary_column(line: &str, column: usize) -> usize {
    for (start, end) in grapheme_column_ranges(line) {
        if start < column && column < end {
            return start;
        }
    }
    column
}

fn grapheme_range_end_boundary_column(line: &str, column: usize) -> usize {
    for (start, end) in grapheme_column_ranges(line) {
        if start < column && column < end {
            return end;
        }
    }
    column
}

fn grapheme_column_ranges(line: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    line.grapheme_indices(true).map(|(byte_index, grapheme)| {
        let start = line[..byte_index].chars().count();
        let end = start + grapheme.chars().count();
        (start, end)
    })
}

fn grapheme_word_columns(line: &str) -> Vec<GraphemeColumn> {
    line.grapheme_indices(true)
        .map(|(byte_index, grapheme)| {
            let start = line[..byte_index].chars().count();
            let end = start + grapheme.chars().count();
            GraphemeColumn {
                start,
                end,
                is_word: grapheme.chars().any(is_word_character),
            }
        })
        .collect()
}

fn grapheme_word_index_for_column(
    line: &str,
    column: usize,
) -> Result<(Vec<GraphemeColumn>, usize), BufferError> {
    let boundary = nearest_grapheme_boundary_column(line, column)?;
    let units = grapheme_word_columns(line);
    let index = units
        .iter()
        .position(|unit| unit.start >= boundary)
        .unwrap_or(units.len());
    Ok((units, index))
}
