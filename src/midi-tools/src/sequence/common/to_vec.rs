use std::iter::FromIterator;

/// Converts an iterator into a vector.
///
/// Useful when you to cache the result of an iterator for future use.
///
/// Very similar to the to_vec crate
///```
///use midi_tools::{events::Event, pipe, sequence::to_vec};
///
///let events = vec![
///    Event::new_note_on_event(100.0f64, 0, 64, 127),
///    Event::new_note_off_event(50.0f64, 0, 64),
///    Event::new_note_on_event(30.0f64, 0, 64, 127),
///    Event::new_note_off_event(80.0f64, 0, 64),
///];
///
///let collected = pipe! { events.into_iter()|>to_vec() };
///
///assert_eq!(
///    collected,
///    vec![
///        Event::new_note_on_event(100.0f64, 0, 64, 127),
///        Event::new_note_off_event(50.0f64, 0, 64),
///        Event::new_note_on_event(30.0f64, 0, 64, 127),
///        Event::new_note_off_event(80.0f64, 0, 64),
///    ]
///)
///```
pub fn to_vec<T, I: Iterator<Item = T> + Sized>(iter: I) -> Vec<T> {
    FromIterator::from_iter(iter)
}

#[cfg(test)]
mod tests {
    use crate::{events::Event, pipe, sequence::to_vec};

    #[test]
    fn test() {
        let events = vec![
            Event::new_note_on_event(100.0f64, 0, 64, 127),
            Event::new_note_off_event(50.0f64, 0, 64),
            Event::new_note_on_event(30.0f64, 0, 64, 127),
            Event::new_note_off_event(80.0f64, 0, 64),
        ];

        let collected = pipe! { events.into_iter()|>to_vec() };

        assert_eq!(
            collected,
            vec![
                Event::new_note_on_event(100.0f64, 0, 64, 127),
                Event::new_note_off_event(50.0f64, 0, 64),
                Event::new_note_on_event(30.0f64, 0, 64, 127),
                Event::new_note_off_event(80.0f64, 0, 64),
            ]
        )
    }
}
