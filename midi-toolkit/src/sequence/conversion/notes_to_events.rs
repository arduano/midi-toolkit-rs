use std::collections::BinaryHeap;

use crate::gen_iter::GenIter;

use crate::{
    events::{Event, NoteOffEvent},
    notes::MIDINote,
    num::MIDINum,
    sequence::event::Delta,
    unwrap,
};

/// A temporary struct for ordering note off events in a binary heap.
struct NoteOffHolder<D: MIDINum>(Delta<D, NoteOffEvent>);

impl<D: MIDINum> NoteOffHolder<D> {
    fn new(delta: D, event: NoteOffEvent) -> Self {
        Self(Delta::new(delta, event))
    }

    fn into_event(self) -> Delta<D, Event> {
        Delta::new(self.0.delta, Event::NoteOff(self.0.event))
    }
}

impl<D: MIDINum> PartialEq for NoteOffHolder<D> {
    fn eq(&self, other: &Self) -> bool {
        self.0.delta == other.0.delta
    }
}
impl<D: MIDINum> Eq for NoteOffHolder<D> {}

impl<D: MIDINum> Ord for NoteOffHolder<D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .delta
            .partial_cmp(&other.0.delta)
            .unwrap_or(std::cmp::Ordering::Equal)
            .reverse()
    }
}

impl<D: MIDINum> PartialOrd for NoteOffHolder<D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Takes a note iterator and converts it to a note event iterator.
/// Effectively flattening the notes into an event sequence.
pub fn notes_to_events<D: MIDINum, N: MIDINote<D>, Err>(
    iter: impl Iterator<Item = Result<N, Err>> + Sized,
) -> impl Iterator<Item = Result<Delta<D, Event>, Err>> {
    GenIter(
        #[coroutine]
        move || {
            let mut note_offs = BinaryHeap::<NoteOffHolder<D>>::new();

            let mut prev_time = D::zero();

            for note in iter {
                let note = unwrap!(note);

                while let Some(e) = note_offs.peek() {
                    if e.0.delta <= note.start() {
                        let holder = note_offs.pop().unwrap();
                        let mut e = holder.into_event();
                        let time = e.delta;
                        e.delta -= prev_time;
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
                let off = NoteOffEvent::new(note.channel(), note.key());
                let holder = NoteOffHolder::new(time, off);

                note_offs.push(holder);
            }

            while let Some(holder) = note_offs.pop() {
                let mut e = holder.into_event();
                let time = e.delta;
                e.delta -= prev_time;
                prev_time = time;
                yield Ok(e);
            }
        },
    )
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
