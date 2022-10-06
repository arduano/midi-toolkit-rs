use std::collections::VecDeque;

use gen_iter::GenIter;

use crate::{events::Event, notes::MIDINote, num::MIDINum, sequence::event::Delta, unwrap};

/// Takes a note iterator and converts it to a note event iterator.
/// Effectively flattening the notes into an event sequence.
pub fn notes_to_events<D: MIDINum, N: MIDINote<D>, Err>(
    iter: impl Iterator<Item = Result<N, Err>> + Sized,
) -> impl Iterator<Item = Result<Delta<D, Event>, Err>> {
    GenIter(move || {
        let mut note_offs = VecDeque::<Delta<D, Event>>::new();

        let mut prev_time = D::zero();

        for note in iter {
            let note = unwrap!(note);

            while let Some(e) = note_offs.front() {
                if e.delta <= note.start() {
                    let mut e = note_offs.pop_front().unwrap();
                    let time = e.delta;
                    e.delta = e.delta - prev_time;
                    prev_time = time;
                    yield Ok(e);
                } else {
                    break;
                }
            }

            yield Ok(Event::new_delta_note_on_event(
                note.start() - prev_time,
                note.channel(),
                note.key(),
                note.velocity(),
            ));

            prev_time = note.start();

            let time = note.end();
            let off = Event::new_delta_note_off_event(time, note.channel(), note.key());

            if note_offs.len() == 0 {
                note_offs.push_back(off);
            } else {
                // binary search

                let len = note_offs.len();
                let mut pos = len / 2;
                let mut jump = note_offs.len() / 4;
                loop {
                    if jump < 1 {
                        jump = 1;
                    }

                    let e = &note_offs[pos];

                    if e.delta >= time {
                        if pos == 0 || note_offs[pos - 1].delta < time {
                            note_offs.insert(pos, off);
                            break;
                        } else {
                            if jump > pos {
                                pos = 0;
                            } else {
                                pos -= jump;
                            }
                        }
                    } else {
                        if pos == len - 1 {
                            note_offs.push_back(off);
                            break;
                        } else {
                            if jump >= len - pos {
                                pos = len - 1;
                            } else {
                                pos += jump;
                            }
                        }
                    }

                    jump /= 2;
                }
            }
        }

        while let Some(mut e) = note_offs.pop_front() {
            let time = e.delta;
            e.delta = e.delta - prev_time;
            prev_time = time;
            yield Ok(e);
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        events::Event,
        notes::Note,
        pipe,
        sequence::{conversion::notes_to_events, to_vec_result, wrap_ok},
    };

    #[test]
    fn convert_notes_to_events() {
        let events = vec![
            Note {
                start: 100.0f64,
                channel: 0,
                key: 64,
                velocity: 127,
                len: 80.0,
            },
            Note {
                start: 130.0,
                channel: 0,
                key: 64,
                velocity: 127,
                len: 130.0,
            },
            Note {
                start: 260.0,
                channel: 1,
                key: 64,
                velocity: 127,
                len: 80.0,
            },
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>notes_to_events()
            |>to_vec_result().unwrap()
        };

        let expected = vec![
            Event::new_delta_note_on_event(100.0f64, 0, 64, 127),
            Event::new_delta_note_on_event(30.0f64, 0, 64, 127),
            Event::new_delta_note_off_event(50.0f64, 0, 64),
            Event::new_delta_note_off_event(80.0f64, 0, 64),
            Event::new_delta_note_on_event(0.0f64, 1, 64, 127),
            Event::new_delta_note_off_event(80.0f64, 1, 64),
        ];

        assert_eq!(changed, expected);
    }
}
