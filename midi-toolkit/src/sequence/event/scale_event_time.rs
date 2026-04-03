use crate::{events::MIDIDelta, num::MIDINum};

/// Scale each delta time of an event sequence.
///
/// Similar to [`scale_event_ppq`](crate::sequence::event::scale_event_ppq), except only takes the multiplier.
/// ## Example
///```
///use midi_toolkit::{events::Event, prelude::*};
///
///let events = vec![
///    Event::new_delta_note_on_event(100.0f64, 0, 64, 127),
///    Event::new_delta_note_off_event(50.0f64, 0, 64),
///    Event::new_delta_note_on_event(30.0f64, 0, 64, 127),
///    Event::new_delta_note_off_event(80.0f64, 0, 64),
///];
///
///let changed = events
///    .into_iter()
///    .into_ok()
///    .scale_event_time(1.5)
///    .collect_vec_result()
///    .unwrap();
///
///assert_eq!(
///    changed,
///    vec![
///        Event::new_delta_note_on_event(150.0f64, 0, 64, 127),
///        Event::new_delta_note_off_event(75.0f64, 0, 64),
///        Event::new_delta_note_on_event(45.0f64, 0, 64, 127),
///        Event::new_delta_note_off_event(120.0f64, 0, 64),
///    ]
///)
///```
pub fn scale_event_time<
    D: MIDINum,
    E: MIDIDelta<D>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    iter: I,
    multiplier: D,
) -> impl Iterator<Item = Result<E, Err>> {
    iter.map(move |e| {
        let mut e = e?;
        let delta = e.delta_mut();
        *delta *= multiplier;
        Ok(e)
    })
}

#[cfg(test)]
mod tests {
    use crate::{events::Event, prelude::*};

    #[test]
    fn time_change() {
        let events = vec![
            Event::new_delta_note_on_event(100.0f64, 0, 64, 127),
            Event::new_delta_note_off_event(50.0f64, 0, 64),
            Event::new_delta_note_on_event(30.0f64, 0, 64, 127),
            Event::new_delta_note_off_event(80.0f64, 0, 64),
        ];

        let changed = events
            .into_iter()
            .into_ok()
            .scale_event_time(1.5)
            .collect_vec_result()
            .unwrap();

        assert_eq!(
            changed,
            vec![
                Event::new_delta_note_on_event(150.0f64, 0, 64, 127),
                Event::new_delta_note_off_event(75.0f64, 0, 64),
                Event::new_delta_note_on_event(45.0f64, 0, 64, 127),
                Event::new_delta_note_off_event(120.0f64, 0, 64),
            ]
        )
    }

    #[test]
    fn time_change_ints() {
        let events = vec![
            Event::new_delta_note_on_event(100, 0, 64, 127),
            Event::new_delta_note_off_event(50, 0, 64),
            Event::new_delta_note_on_event(30, 0, 64, 127),
            Event::new_delta_note_off_event(80, 0, 64),
        ];

        let changed = events
            .into_iter()
            .into_ok()
            .scale_event_time(2)
            .collect_vec_result()
            .unwrap();

        assert_eq!(
            changed,
            vec![
                Event::new_delta_note_on_event(200, 0, 64, 127),
                Event::new_delta_note_off_event(100, 0, 64),
                Event::new_delta_note_on_event(60, 0, 64, 127),
                Event::new_delta_note_off_event(160, 0, 64),
            ]
        )
    }
}
