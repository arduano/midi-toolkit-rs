/// Wraps each item `T` into `Result<T, ()>`
///
/// Useful because all built in functions use Result as the item type for error handling
/// ## Example
///```
///use midi_tools::{events::Event, pipe, sequence::{to_vec, wrap_ok}};
///    
///let events = vec![
///    Event::new_note_on_event(100, 0, 64, 127),
///    Event::new_note_off_event(50, 0, 64),
///];
///
///let changed = pipe! { events.into_iter()|>wrap_ok()|>to_vec() };
///
///assert_eq!(
///    changed,
///    vec![
///        Ok(Event::new_note_on_event(100, 0, 64, 127)),
///        Ok(Event::new_note_off_event(50, 0, 64)),
///    ]
///)
///```
pub fn wrap_ok<T, I: Iterator<Item = T> + Sized>(iter: I) -> impl Iterator<Item = Result<T, ()>> {
    iter.map(|v| Ok(v))
}
