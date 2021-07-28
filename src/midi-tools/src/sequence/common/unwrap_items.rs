
use std::fmt::Debug;

/// Unwraps each item `Result<T, E>` into T, panicking if an error is reached
pub fn unwrap_items<T: Debug, E: Debug, I: Iterator<Item = Result<T, E>> + Sized>(
    iter: I,
) -> impl Iterator<Item = T> {
    iter.map(|v| v.unwrap())
}

#[cfg(test)]
mod tests {
    use crate::{events::Event, pipe, sequence::{to_vec, unwrap_items}};

    #[test]
    #[should_panic]
    fn panic() {
        let events = vec![
            Ok(Event::new_note_on_event(100.0f64, 0, 64, 127)),
            Ok(Event::new_note_off_event(50.0f64, 0, 64)),
            Err(()),
        ];

        pipe! {
            events.into_iter()
            |>unwrap_items()
            |>to_vec()
        };
    }

    #[test]
    fn no_panic() {
        let events: Vec<Result<_, ()>> = vec![
            Ok(Event::new_note_on_event(100.0f64, 0, 64, 127)),
            Ok(Event::new_note_off_event(50.0f64, 0, 64)),
        ];

        let changed = pipe! {
            events.into_iter()
            |>unwrap_items()
            |>to_vec()
        };

        assert_eq!(
            changed,
            vec![
                Event::new_note_on_event(100.0f64, 0, 64, 127),
                Event::new_note_off_event(50.0f64, 0, 64),
            ]
        )
    }
}
