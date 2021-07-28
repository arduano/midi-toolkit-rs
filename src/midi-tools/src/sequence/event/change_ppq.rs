use gen_iter::GenIter;

use crate::{events::MIDIEvent, num::MIDINum, unwrap};

/// Change the PPQ of an event sequence.
///
/// Similar to [`scale_time`](crate::sequence::event::scale_time), except does `new_delta = old_delta * to / from`.
/// ## Example
///```
///use midi_tools::{
///    events::Event,
///    pipe,
///    sequence::{event::change_ppq, to_vec_result, wrap_ok},
///};
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
///    |>change_ppq(64.0, 96.0)
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
pub fn change_ppq<T: MIDINum, E: MIDIEvent<T>, Err, I: Iterator<Item = Result<E, Err>> + Sized>(
    iter: I,
    from: T,
    to: T,
) -> impl Iterator<Item = Result<E, Err>> {
    GenIter(move || {
        for e in iter {
            let mut e = unwrap!(e);
            let delta = e.delta_mut();
            *delta = *delta * to / from;
            yield Ok(e);
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        events::Event,
        pipe,
        sequence::{event::change_ppq, to_vec_result, wrap_ok},
    };

    #[test]
    fn delta_change() {
        let events = vec![
            Event::new_note_on_event(100.0f64, 0, 64, 127),
            Event::new_note_off_event(50.0f64, 0, 64),
            Event::new_note_on_event(30.0f64, 0, 64, 127),
            Event::new_note_off_event(80.0f64, 0, 64),
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>change_ppq(64.0, 96.0)
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
    fn delta_change_ints() {
        let events = vec![
            Event::new_note_on_event(100, 0, 64, 127),
            Event::new_note_off_event(50, 0, 64),
            Event::new_note_on_event(30, 0, 64, 127),
            Event::new_note_off_event(80, 0, 64),
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>change_ppq(64, 96)
            |>to_vec_result().unwrap()
        };

        assert_eq!(
            changed,
            vec![
                Event::new_note_on_event(150, 0, 64, 127),
                Event::new_note_off_event(75, 0, 64),
                Event::new_note_on_event(45, 0, 64, 127),
                Event::new_note_off_event(120, 0, 64),
            ]
        )
    }

    #[test]
    fn delta_change_ints_divide() {
        let events = vec![
            Event::new_note_on_event(100, 0, 64, 127),
            Event::new_note_off_event(50, 0, 64),
            Event::new_note_on_event(30, 0, 64, 127),
            Event::new_note_off_event(80, 0, 64),
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>change_ppq(3, 2)
            |>to_vec_result().unwrap()
        };

        assert_eq!(
            changed,
            vec![
                Event::new_note_on_event(66, 0, 64, 127),
                Event::new_note_off_event(33, 0, 64),
                Event::new_note_on_event(20, 0, 64, 127),
                Event::new_note_off_event(53, 0, 64),
            ]
        )
    }
}
