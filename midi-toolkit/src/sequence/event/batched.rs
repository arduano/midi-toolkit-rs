use crate::gen_iter::GenIter;

use crate::{
    events::{BatchTempo, MIDIDelta, MIDIEvent, MIDIEventEnum},
    num::MIDINum,
    unwrap,
};

use super::{Delta, Track};

#[derive(Debug)]
pub struct EventBatch<T> {
    events: Vec<T>,
}

impl<T> EventBatch<T> {
    fn new(events: Vec<T>) -> Self {
        Self { events }
    }

    pub fn into_iter_inner(self) -> impl Iterator<Item = T> {
        self.events.into_iter()
    }

    pub fn iter_inner(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    pub fn count(&self) -> usize {
        self.events.len()
    }
}

impl<D: MIDINum, T> Delta<D, EventBatch<T>> {
    pub fn into_iter_events(self) -> impl Iterator<Item = Delta<D, T>> {
        let mut delta = self.delta;
        self.event.into_iter_inner().map(move |event| {
            let event = Delta::new(delta, event);
            delta = D::zero();
            event
        })
    }

    pub fn iter_events(&self) -> impl Iterator<Item = Delta<D, &T>> {
        let mut delta = self.delta;
        self.event.iter_inner().map(move |event| {
            let event = Delta::new(delta, event);
            delta = D::zero();
            event
        })
    }
}

impl<D: MIDINum, T> Delta<D, Track<EventBatch<T>>> {
    pub fn iter_events(&self) -> impl Iterator<Item = Delta<D, Track<&T>>> {
        let mut delta = self.delta;
        let track = self.event.track;
        self.event.event.iter_inner().map(move |event| {
            let event = Delta::new(delta, Track::new(event, track));
            delta = D::zero();
            event
        })
    }
}

impl<D: MIDINum, T> IntoIterator for Delta<D, Track<EventBatch<T>>> {
    type Item = Delta<D, Track<T>>;
    type IntoIter = impl Iterator<Item = Delta<D, Track<T>>>;

    fn into_iter(self) -> Self::IntoIter {
        let mut delta = self.delta;
        let track = self.event.track;
        self.event
            .inner_event()
            .into_iter_inner()
            .map(move |event| {
                let event = Delta::new(delta, Track::new(event, track));
                delta = D::zero();
                event
            })
    }
}

impl<E: MIDIEventEnum> BatchTempo for EventBatch<E> {
    fn inner_tempo(&self) -> Option<u32> {
        for e in self.events.iter().rev() {
            if let Some(t) = e.as_event().inner_tempo() {
                return Some(t);
            }
        }
        None
    }

    fn without_tempo(self) -> Option<Self> {
        let new = self
            .events
            .into_iter()
            .filter(|e| e.as_event().inner_tempo().is_some())
            .collect::<Vec<_>>();

        if new.is_empty() {
            None
        } else {
            Some(Self::new(new))
        }
    }
}

pub fn convert_events_into_batches<D: MIDINum, E, Err>(
    iter: impl Iterator<Item = Result<Delta<D, E>, Err>>,
) -> impl Iterator<Item = Result<Delta<D, EventBatch<E>>, Err>> {
    GenIter(
        #[coroutine]
        move || {
            let mut next_batch = Delta::new(D::zero(), EventBatch::new(Vec::new()));
            for e in iter {
                let e = unwrap!(e);
                if e.delta() > D::zero() {
                    if !next_batch.events.is_empty() {
                        yield Ok(next_batch);
                    }
                    next_batch = Delta::new(e.delta(), EventBatch::new(Vec::new()));
                }
                next_batch.events.push(e.event);
            }
            if !next_batch.events.is_empty() {
                yield Ok(next_batch);
            }
        },
    )
}

pub fn flatten_batches_to_events<D: MIDINum, E: MIDIEvent, Err>(
    iter: impl Iterator<Item = Result<Delta<D, EventBatch<E>>, Err>>,
) -> impl Iterator<Item = Result<Delta<D, E>, Err>> {
    GenIter(
        #[coroutine]
        move || {
            for batch in iter {
                let batch = unwrap!(batch);
                let mut delta = batch.delta;
                for event in batch.event.into_iter_inner() {
                    yield Ok(Delta::new(delta, event));
                    delta = D::zero();
                }
            }
        },
    )
}

pub fn flatten_track_batches_to_events<D: MIDINum, E: MIDIEvent, Err>(
    iter: impl Iterator<Item = Result<Delta<D, Track<EventBatch<E>>>, Err>>,
) -> impl Iterator<Item = Result<Delta<D, Track<E>>, Err>> {
    GenIter(
        #[coroutine]
        move || {
            for batch in iter {
                let batch = unwrap!(batch);
                let track = batch.event.track;
                let mut delta = batch.delta;
                for event in batch.event.inner_event().into_iter_inner() {
                    yield Ok(Delta::new(delta, Track::new(event, track)));
                    delta = D::zero();
                }
            }
        },
    )
}
