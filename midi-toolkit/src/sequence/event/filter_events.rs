use gen_iter::GenIter;

use crate::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    num::MIDINum,
    unwrap,
};

/// Filter the events in a sequence based on a predicate, while carrying over the delta of the removed events.
///
/// ## Example
///```
/// use midi_toolkit::{
///     events::Event,
///     pipe,
///     sequence::{event::filter_events, to_vec_result, wrap_ok},
/// };
///
/// let events = vec![
///     Event::new_note_on_event(100.0f64, 0, 64, 127),
///     Event::new_note_off_event(50.0f64, 0, 64),
///     Event::new_note_on_event(30.0f64, 0, 64, 127),
///     Event::new_note_off_event(80.0f64, 0, 64),
/// ];
///
/// let changed = pipe! {
///     events.into_iter()
///     |>wrap_ok()
///     |>filter_events(|e| match e {
///         Event::NoteOn { .. } => true,
///         _ => false,
///     })
///     |>to_vec_result().unwrap()
/// };
///
/// assert_eq!(
///     changed,
///     vec![
///         Event::new_note_on_event(100.0f64, 0, 64, 127),
///         Event::new_note_on_event(80.0f64, 0, 64, 127),
///     ]
/// )
///```
pub fn filter_events<T, E, Err, I>(
    iter: I,
    predicate: impl Fn(&E) -> bool,
) -> impl Iterator<Item = Result<E, Err>>
where
    T: MIDINum,
    E: MIDIEvent<T>,
    I: Iterator<Item = Result<E, Err>> + Sized,
{
    let mut extra_delta = T::zero();
    GenIter(move || {
        for e in iter {
            let mut e = unwrap!(e);
            if predicate(&e) {
                e.set_delta(e.delta() + extra_delta);
                extra_delta = T::zero();
                yield Ok(e);
            } else {
                extra_delta += e.delta();
            }
        }
    })
}

/// Similar to [`filter_events`](crate::sequence::event::filter_events), except keeps only note on and note off events.
///
/// ## Example
///```
/// use midi_toolkit::{
///     events::Event,
///     pipe,
///     sequence::{event::filter_note_events, to_vec_result, wrap_ok},
/// };
///
/// let events = vec![
///     Event::new_note_on_event(100.0f64, 0, 64, 127),
///     Event::new_channel_pressure_event(50.0f64, 0, 64),
///     Event::new_note_on_event(30.0f64, 0, 64, 127),
///     Event::new_channel_pressure_event(80.0f64, 0, 64),
/// ];
///
/// let changed = pipe! {
///     events.into_iter()
///     |>wrap_ok()
///     |>filter_note_events()
///     |>to_vec_result().unwrap()
/// };
///
/// assert_eq!(
///     changed,
///     vec![
///         Event::new_note_on_event(100.0f64, 0, 64, 127),
///         Event::new_note_on_event(80.0f64, 0, 64, 127),
///     ]
/// )
///```
pub fn filter_note_events<T, E, Err, I>(iter: I) -> impl Iterator<Item = Result<E, Err>>
where
    T: MIDINum,
    E: MIDIEventEnum<T>,
    I: Iterator<Item = Result<E, Err>> + Sized,
{
    filter_events(iter, |e| match e.as_event() {
        Event::NoteOn { .. } | Event::NoteOff { .. } => true,
        _ => false,
    })
}

/// Similar to [`filter_events`](crate::sequence::event::filter_events), except removes only note on and note off events.
///
/// ## Example
///```
/// use midi_toolkit::{
///     events::Event,
///     pipe,
///     sequence::{event::filter_non_note_events, to_vec_result, wrap_ok},
/// };
///
/// let events = vec![
///     Event::new_note_on_event(100.0f64, 0, 64, 127),
///     Event::new_channel_pressure_event(50.0f64, 0, 64),
///     Event::new_note_on_event(30.0f64, 0, 64, 127),
///     Event::new_channel_pressure_event(80.0f64, 0, 64),
/// ];
///
/// let changed = pipe! {
///     events.into_iter()
///     |>wrap_ok()
///     |>filter_non_note_events()
///     |>to_vec_result().unwrap()
/// };
///
/// assert_eq!(
///     changed,
///     vec![
///         Event::new_channel_pressure_event(150.0f64, 0, 64),
///         Event::new_channel_pressure_event(110.0f64, 0, 64),
///     ]
/// )
///```
pub fn filter_non_note_events<T, E, Err, I>(iter: I) -> impl Iterator<Item = Result<E, Err>>
where
    T: MIDINum,
    E: MIDIEventEnum<T>,
    I: Iterator<Item = Result<E, Err>> + Sized,
{
    filter_events(iter, |e| match e.as_event() {
        Event::NoteOn { .. } | Event::NoteOff { .. } => false,
        _ => true,
    })
}
