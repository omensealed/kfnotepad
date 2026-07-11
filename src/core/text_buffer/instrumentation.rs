#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static TO_TEXT_CALL_COUNT: Cell<usize> = const { Cell::new(0) };
    static FROM_TEXT_CALL_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn reset_to_text_call_count() {
    TO_TEXT_CALL_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn to_text_call_count() -> usize {
    TO_TEXT_CALL_COUNT.with(Cell::get)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn reset_from_text_call_count() {
    FROM_TEXT_CALL_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn from_text_call_count() -> usize {
    FROM_TEXT_CALL_COUNT.with(Cell::get)
}
