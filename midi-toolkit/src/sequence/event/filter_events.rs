use crate::gen_iter::GenIter;

use crate::{
    events::{Event, MIDIDelta, MIDIEventEnum},
    num::MIDINum,
    sequence::event::Delta,
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

/// Filter-map the events in a sequence based on an owned event mapper, while carrying over the
/// delta of removed events onto the next emitted event.
pub fn filter_map_events<D, E, NE, Err, I>(
    iter: I,
    mut mapper: impl FnMut(E) -> Option<NE>,
) -> impl Iterator<Item = Result<Delta<D, NE>, Err>>
where
    D: MIDINum,
    I: Iterator<Item = Result<Delta<D, E>, Err>> + Sized,
{
    let mut extra_delta = D::zero();
    GenIter(
        #[coroutine]
        move || {
            for e in iter {
                let Delta { delta, event } = unwrap!(e);
                if let Some(mapped) = mapper(event) {
                    yield Ok(Delta::new(delta + extra_delta, mapped));
                    extra_delta = D::zero();
                } else {
                    extra_delta += delta;
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

#[cfg(test)]
mod tests {
    use super::filter_map_events;
    use crate::{
        events::{Event, NoteOffEvent, NoteOnEvent},
        sequence::event::Delta,
        sequence::ResultIterExt,
    };

    #[test]
    fn filter_map_events_carries_removed_delta_to_next_mapped_item() {
        let events: Vec<Result<Delta<u64, Event>, ()>> = vec![
            Ok(Delta::new(
                10_u64,
                Event::NoteOn(NoteOnEvent {
                    channel: 0,
                    key: 60,
                    velocity: 100,
                }),
            )),
            Ok(Delta::new(
                20_u64,
                Event::NoteOff(NoteOffEvent {
                    channel: 0,
                    key: 60,
                }),
            )),
            Ok(Delta::new(
                30_u64,
                Event::NoteOn(NoteOnEvent {
                    channel: 1,
                    key: 64,
                    velocity: 110,
                }),
            )),
        ];

        let mapped: Vec<_> = filter_map_events(events.into_iter(), |event| match event {
            Event::NoteOn(note) => Some((note.channel, note.key)),
            Event::NoteOff(_) => None,
            _ => None,
        })
        .unwrap_items()
        .collect();

        assert_eq!(
            mapped,
            vec![Delta::new(10_u64, (0, 60)), Delta::new(50_u64, (1, 64))]
        );
    }

    #[test]
    fn filter_map_events_can_drop_everything() {
        let events: Vec<Result<Delta<u64, Event>, ()>> = vec![
            Ok(Delta::new(
                12_u64,
                Event::NoteOff(NoteOffEvent {
                    channel: 0,
                    key: 60,
                }),
            )),
            Ok(Delta::new(
                24_u64,
                Event::NoteOff(NoteOffEvent {
                    channel: 1,
                    key: 62,
                }),
            )),
        ];

        let mapped: Vec<Delta<u64, u8>> = filter_map_events(events.into_iter(), |_event| None)
            .unwrap_items()
            .collect();
        assert!(mapped.is_empty());
    }
}
