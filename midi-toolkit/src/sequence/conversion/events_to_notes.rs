use std::{cell::UnsafeCell, collections::VecDeque, rc::Rc};

use gen_iter::GenIter;

use crate::{
    events::{Event, MIDIEventEnum},
    notes::Note,
    num::MIDINum,
    unwrap,
};

// ==============
// Helper structs
// ==============

#[derive(Debug)]
struct UnendedContainer<T> {
    ended: bool,
    note: T,
}

impl<T> UnendedContainer<T> {
    fn new(note: T) -> Self {
        Self { ended: false, note }
    }
}

// Using UnsafeCell for extra performance, because this algorithm is tested and fully safe.
#[derive(Debug)]
struct Shared<T>(Rc<UnsafeCell<T>>);

impl<T> Shared<T> {
    fn new(val: T) -> Self {
        Self(Rc::new(UnsafeCell::new(val)))
    }

    fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }

    fn into_inner(self) -> T {
        Rc::try_unwrap(self.0).unwrap().into_inner()
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct NoteQueue<T: MIDINum> {
    queue: VecDeque<Shared<UnendedContainer<Note<T>>>>,
    keys: Vec<VecDeque<Shared<UnendedContainer<Note<T>>>>>,
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
    fn get_queue(
        &mut self,
        key: u8,
        channel: u8,
    ) -> &mut VecDeque<Shared<UnendedContainer<Note<T>>>> {
        &mut self.keys[key as usize * 16 + channel as usize]
    }

    #[inline(always)]
    fn push(&mut self, note: Note<T>) {
        let key = self.get_queue(note.key, note.channel);
        let note = Shared::new(UnendedContainer::new(note));
        key.push_back(note.clone());
        self.queue.push_back(note);
    }

    #[inline(always)]
    fn end_next(&mut self, key: u8, channel: u8, end: T) {
        let queue = self.get_queue(key, channel);
        if let Some(note) = queue.pop_front() {
            let note = note.get();
            note.ended = true;
            note.note.len = end - note.note.start;
        }
    }

    #[inline(always)]
    fn end_all(&mut self, end: T) {
        for key in self.keys.iter_mut() {
            for note in key.into_iter() {
                let note = note.get();
                note.ended = true;
                note.note.len = end - note.note.start;
            }
        }
    }

    #[inline(always)]
    fn next_ended_note(&mut self) -> Option<Note<T>> {
        let next = self.queue.front();
        if let Some(next) = next {
            if next.get().ended {
                return Some(self.queue.pop_front().unwrap().into_inner().note);
            }
        }
        return None;
    }
}

/// Takes an event iterator and converts it to a note iterator.
/// Effectively extracting the notes from an event sequence.
pub fn events_to_notes<
    T: MIDINum,
    E: MIDIEventEnum<T>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    iter: I,
) -> impl Iterator<Item = Result<Note<T>, Err>> {
    GenIter(move || {
        let mut note_queue = NoteQueue::<T>::new();

        let mut time = T::zero();
        for e in iter {
            let e = unwrap!(e);

            time += e.delta();
            match e.as_event() {
                Event::NoteOn(e) => {
                    let note = Note {
                        start: time,
                        channel: e.channel,
                        key: e.key,
                        velocity: e.velocity,
                        len: T::zero(),
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
    })
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
            Event::new_note_on_event(100.0f64, 0, 64, 127),
            Event::new_note_on_event(30.0f64, 0, 64, 127),
            Event::new_tempo_event(25.0f64, 0),
            Event::new_note_off_event(25.0f64, 0, 64),
            Event::new_note_off_event(80.0f64, 0, 64),
            Event::new_note_on_event(0.0f64, 1, 64, 127),
            Event::new_note_off_event(80.0f64, 1, 64),
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
