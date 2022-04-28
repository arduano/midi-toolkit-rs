use std::marker::PhantomData;

use gen_iter::GenIter;

use crate::{
    events::{ChannelEvent, KeyEvent, MIDIEvent, PlaybackEvent, SerializeEvent},
    io::MIDIWriteError,
    num::MIDINum,
    unwrap,
};

use super::TrackEvent;

#[derive(Debug)]
pub struct EventBatch<D: MIDINum, T: MIDIEvent<D>> {
    events: Vec<T>,
    _delta: PhantomData<D>,
}

impl<D: MIDINum, T: MIDIEvent<D>> EventBatch<D, T> {
    fn new(events: Vec<T>) -> Self {
        Self {
            events,
            _delta: PhantomData,
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.events.into_iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }
}

impl<D: MIDINum, T: MIDIEvent<D>> SerializeEvent for EventBatch<D, T> {
    fn serialize_event<W: std::io::Write>(&self, _buf: &mut W) -> Result<usize, MIDIWriteError> {
        let mut written = 0;
        for event in self.iter() {
            written += event.serialize_event(_buf)?;
        }
        Ok(written)
    }
}

impl<D: MIDINum, T: MIDIEvent<D>> MIDIEvent<D> for EventBatch<D, T> {
    fn delta(&self) -> D {
        self.events[0].delta()
    }

    fn delta_mut(&mut self) -> &mut D {
        self.events[0].delta_mut()
    }

    fn key(&self) -> Option<u8> {
        None
    }

    fn key_mut(&mut self) -> Option<&mut u8> {
        None
    }

    fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>> {
        None
    }

    fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>> {
        None
    }

    fn channel(&self) -> Option<u8> {
        None
    }

    fn channel_mut(&mut self) -> Option<&mut u8> {
        None
    }

    fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>> {
        None
    }

    fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>> {
        None
    }

    fn as_u32(&self) -> Option<u32> {
        None
    }

    fn as_playback_event<'a>(&'a self) -> Option<Box<&'a dyn PlaybackEvent>> {
        None
    }
}

pub fn convert_events_into_batches<D: MIDINum, E: MIDIEvent<D>, Err>(
    iter: impl Iterator<Item = Result<E, Err>>,
) -> impl Iterator<Item = Result<EventBatch<D, E>, Err>> {
    GenIter(move || {
        let mut events = Vec::new();
        for e in iter {
            let e = unwrap!(e);
            if e.delta() > D::zero() {
                if events.len() > 0 {
                    yield Ok(EventBatch::new(events));
                }
                events = Vec::new();
            }
            events.push(e);
        }
    })
}

pub fn flatten_batches_to_events<D: MIDINum, E: MIDIEvent<D>, Err>(
    iter: impl Iterator<Item = Result<EventBatch<D, E>, Err>>,
) -> impl Iterator<Item = Result<E, Err>> {
    GenIter(move || {
        for batch in iter {
            let batch = unwrap!(batch);
            for event in batch.into_iter() {
                yield Ok(event);
            }
        }
    })
}

pub fn flatten_track_batches_to_events<D: MIDINum, E: MIDIEvent<D>, Err>(
    iter: impl Iterator<Item = Result<TrackEvent<D, EventBatch<D, E>>, Err>>,
) -> impl Iterator<Item = Result<TrackEvent<D, E>, Err>> {
    GenIter(move || {
        for batch in iter {
            let batch = unwrap!(batch);
            let track = batch.track;
            for event in batch.into().into_iter() {
                yield Ok(TrackEvent::new(event, track));
            }
        }
    })
}
