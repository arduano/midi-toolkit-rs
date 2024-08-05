use crate::gen_iter::GenIter;

use crate::{
    events::{Event, MIDIDelta, MIDIEventEnum},
    num::MIDINum,
    unwrap,
};

/// Filter the events in a sequence based on a predicate, while carrying over the delta of the removed events.
pub fn filter_events<D, E, Err, I>(
    iter: I,
    predicate: impl Fn(&E) -> bool,
) -> impl Iterator<Item = Result<E, Err>>
where
    D: MIDINum,
    E: MIDIEventEnum + MIDIDelta<D>,
    I: Iterator<Item = Result<E, Err>> + Sized,
{
    let mut extra_delta = D::zero();
    GenIter(
        #[coroutine]
        move || {
            for e in iter {
                let mut e = unwrap!(e);
                if predicate(&e) {
                    e.set_delta(e.delta() + extra_delta);
                    extra_delta = D::zero();
                    yield Ok(e);
                } else {
                    extra_delta += e.delta();
                }
            }
        },
    )
}

/// Similar to [`filter_events`](crate::sequence::event::filter_events), except keeps only note on and note off events.
pub fn filter_note_events<D, E, Err, I>(iter: I) -> impl Iterator<Item = Result<E, Err>>
where
    D: MIDINum,
    E: MIDIEventEnum + MIDIDelta<D>,
    I: Iterator<Item = Result<E, Err>> + Sized,
{
    filter_events(iter, |e| {
        matches!(e.as_event(), Event::NoteOn { .. } | Event::NoteOff { .. })
    })
}

/// Similar to [`filter_events`](crate::sequence::event::filter_events), except removes only note on and note off events.
pub fn filter_non_note_events<D, E, Err, I>(iter: I) -> impl Iterator<Item = Result<E, Err>>
where
    D: MIDINum,
    E: MIDIEventEnum + MIDIDelta<D>,
    I: Iterator<Item = Result<E, Err>> + Sized,
{
    filter_events(iter, |e| {
        !matches!(e.as_event(), Event::NoteOn { .. } | Event::NoteOff { .. })
    })
}
