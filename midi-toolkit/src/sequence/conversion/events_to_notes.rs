use std::{cell::Cell, collections::VecDeque, rc::Rc};

use gen_iter::GenIter;

use crate::{
    events::{Event, MIDIEventEnum},
    notes::Note,
    num::MIDINum,
    sequence::event::Delta,
    unwrap,
};

// ==============
// Helper structs
// ==============

#[derive(Debug)]
struct UnendedContainer<T: MIDINum> {
    new_end: Cell<Option<T>>,
    note: Note<T>,
}

impl<T: MIDINum> UnendedContainer<T> {
    fn new(note: Note<T>) -> Self {
        Self {
            new_end: Cell::new(None),
            note,
        }
    }
}

struct NoteQueue<T: MIDINum> {
    queue: VecDeque<Rc<UnendedContainer<T>>>,
    keys: Vec<VecDeque<Rc<UnendedContainer<T>>>>,
}

impl<T: MIDINum> NoteQueue<T> {
    fn new() -> Self {
        let queue = VecDeque::new();
        let mut keys = Vec::new();
        for _ in 0..(256 * 16) {
            keys.push(VecDeque::new());
        }

        Self { queue, keys }
    }

    #[inline(always)]
    fn get_queue(&mut self, key: u8, channel: u8) -> &mut VecDeque<Rc<UnendedContainer<T>>> {
        &mut self.keys[key as usize * 16 + channel as usize]
    }

    #[inline(always)]
    fn push(&mut self, note: Note<T>) {
        let key = self.get_queue(note.key, note.channel);
        let note = Rc::new(UnendedContainer::new(note));
        key.push_back(note.clone());
        self.queue.push_back(note);
    }

    #[inline(always)]
    fn end_next(&mut self, key: u8, channel: u8, end: T) {
        let queue = self.get_queue(key, channel);
        if let Some(note) = queue.pop_front() {
            note.new_end.set(Some(end));
        }
    }

    #[inline(always)]
    fn end_all(&mut self, end: T) {
        for key in self.keys.iter_mut() {
            for note in key.iter_mut() {
                note.new_end.set(Some(end));
            }
        }
    }

    #[inline(always)]
    fn next_ended_note(&mut self) -> Option<Note<T>> {
        let next = self.queue.front();
        if let Some(next) = next {
            if let Some(end) = next.new_end.get() {
                let next_note = self.queue.pop_front().unwrap();
                let mut note = Rc::try_unwrap(next_note).unwrap().note;
                note.len = end - note.start;
                return Some(note);
            }
        }
        None
    }
}

/// Takes an event iterator and converts it to a note iterator.
/// Effectively extracting the notes from an event sequence.
pub fn events_to_notes<
    D: MIDINum,
    E: MIDIEventEnum,
    Err,
    I: Iterator<Item = Result<Delta<D, E>, Err>> + Sized,
>(
    iter: I,
) -> impl Iterator<Item = Result<Note<D>, Err>> {
    GenIter(
        #[coroutine]
        move || {
            let mut note_queue = NoteQueue::<D>::new();

            let mut time = D::zero();
            for e in iter {
                let e = unwrap!(e);

                time += e.delta;
                match e.as_event() {
                    Event::NoteOn(e) => {
                        let note = Note {
                            start: time,
                            channel: e.channel,
                            key: e.key,
                            velocity: e.velocity,
                            len: D::zero(),
                        };

                        note_queue.push(note);
                    }
                    Event::NoteOff(e) => {
                        note_queue.end_next(e.key, e.channel, time);

                        while let Some(note) = note_queue.next_ended_note() {
                            yield Ok(note);
                        }
                    }
                    _ => {}
                }
            }

            note_queue.end_all(time);
            while let Some(note) = note_queue.next_ended_note() {
                yield Ok(note);
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
        sequence::{conversion::events_to_notes, to_vec_result, wrap_ok},
    };

    #[test]
    fn convert_events_to_notes() {
        let events = vec![
            Event::new_delta_note_on_event(100.0f64, 0, 64, 127),
            Event::new_delta_note_on_event(30.0f64, 0, 64, 127),
            Event::new_delta_tempo_event(25.0f64, 0),
            Event::new_delta_note_off_event(25.0f64, 0, 64),
            Event::new_delta_note_off_event(80.0f64, 0, 64),
            Event::new_delta_note_on_event(0.0f64, 1, 64, 127),
            Event::new_delta_note_off_event(80.0f64, 1, 64),
        ];

        let changed = pipe! {
            events.into_iter()
            |>wrap_ok()
            |>events_to_notes()
            |>to_vec_result().unwrap()
        };

        let expected = vec![
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

        assert_eq!(changed, expected);
    }
}
