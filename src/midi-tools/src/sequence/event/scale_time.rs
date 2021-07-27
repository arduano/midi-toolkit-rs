use gen_iter::GenIter;

use crate::{events::MIDIEvent, num::MIDINum, unwrap};

/// Scale each delta time of an event sequence.
///
/// Similar to scale_time, except only takes the multiplier.
/// ## Example
///```
///use midi_tools::{events::Event, pipe, sequence::{event::scale_time, to_vec_result, wrap_ok}};
///
///let events = vec![
///    Event::new_note_on_event(100.0f64, 0, 64, 127),
///    Event::new_note_off_event(50.0f64, 0, 64),
///    Event::new_note_on_event(30.0f64, 0, 64, 127),
///    Event::new_note_off_event(80.0f64, 0, 64),
///];
///
///let changed = pipe! {
///    events.into_iter()
///    |>wrap_ok()
///    |>scale_time(1.5)
///    |>to_vec_result().unwrap()
///};
///
///assert_eq!(
///    changed,
///    vec![
///        Event::new_note_on_event(150.0f64, 0, 64, 127),
///        Event::new_note_off_event(75.0f64, 0, 64),
///        Event::new_note_on_event(45.0f64, 0, 64, 127),
///        Event::new_note_off_event(120.0f64, 0, 64),
///    ]
///)
///```
pub fn scale_time<T: MIDINum, E: MIDIEvent<T>, Err, I: Iterator<Item = Result<E, Err>> + Sized>(
    iter: I,
    multiplier: T,
) -> impl Iterator<Item = Result<E, Err>> {
    GenIter(move || {
        for e in iter {
            let mut e = unwrap!(e);
            let delta = e.delta_mut();
            *delta = *delta * multiplier;
            yield Ok(e);
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        events::Event,
        pipe,
        sequence::{event::scale_time, to_vec_result, wrap_ok},
    };

    #[test]
    fn time_change() {
        let events = vec![
            Event::new_note_on_event(100.0f64, 0, 64, 127),
            Event::new_note_off_event(50.0f64, 0, 64),
            Event::new_note_on_event(30.0f64, 0, 64, 127),
            Event::new_note_off_event(80.0f64, 0, 64),
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>scale_time(1.5)
            |>to_vec_result().unwrap()
        };

        assert_eq!(
            changed,
            vec![
                Event::new_note_on_event(150.0f64, 0, 64, 127),
                Event::new_note_off_event(75.0f64, 0, 64),
                Event::new_note_on_event(45.0f64, 0, 64, 127),
                Event::new_note_off_event(120.0f64, 0, 64),
            ]
        )
    }

    #[test]
    fn time_change_ints() {
        let events = vec![
            Event::new_note_on_event(100, 0, 64, 127),
            Event::new_note_off_event(50, 0, 64),
            Event::new_note_on_event(30, 0, 64, 127),
            Event::new_note_off_event(80, 0, 64),
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>scale_time(2)
            |>to_vec_result()
            .unwrap()
        };

        assert_eq!(
            changed,
            vec![
                Event::new_note_on_event(200, 0, 64, 127),
                Event::new_note_off_event(100, 0, 64),
                Event::new_note_on_event(60, 0, 64, 127),
                Event::new_note_off_event(160, 0, 64),
            ]
        )
    }
}
