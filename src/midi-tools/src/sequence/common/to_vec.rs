use std::iter::FromIterator;

/// Converts an iterator into a vector.
///
/// Useful when you to cache the result of an iterator for future use.
///
/// Unwraps all results from the iterator items.
/// ## Example
///```
///use midi_tools::{events::Event, pipe, sequence::to_vec};
///
///let events = vec![
///    Ok(Event::new_note_on_event(100.0f64, 0, 64, 127)),
///    Ok(Event::new_note_off_event(50.0f64, 0, 64)),
///    Err(()),
///];
///
///let collected = pipe! { events.into_iter()|>to_vec() };
///
///assert_eq!(
///    collected,
///    vec![
///        Ok(Event::new_note_on_event(100.0f64, 0, 64, 127)),
///        Ok(Event::new_note_off_event(50.0f64, 0, 64)),
///        Err(()),
///    ]
///)
///```
pub fn to_vec<T, I: Iterator<Item = T> + Sized>(iter: I) -> Vec<T> {
    FromIterator::from_iter(iter)
}

/// Converts a result iterator into a vector result.
///
/// Useful when you to cache the result of an iterator for future use.
///
/// Unwraps all results from the iterator items.
/// ## Example
///```
///use midi_tools::{events::Event, pipe, sequence::to_vec_result};
///
///let events = vec![
///    Ok(Event::new_note_on_event(100.0f64, 0, 64, 127)),
///    Ok(Event::new_note_off_event(50.0f64, 0, 64)),
///    Err(()),
///];
///
///let collected = pipe! { events.into_iter()|>to_vec_result() };
///
///assert_eq!(collected, Err(()))
///```
pub fn to_vec_result<T, Err, I: Iterator<Item = Result<T, Err>> + Sized>(
    iter: I,
) -> Result<Vec<T>, Err> {
    FromIterator::from_iter(iter)
}
