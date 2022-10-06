use std::marker::PhantomData;

use crate::{
    events::{
        BatchTempo, ChannelEvent, KeyEvent, MIDIEvent, MIDIEventEnum, PlaybackEvent, SerializeEvent,
    },
    io::MIDIWriteError,
    num::{MIDINum, MIDINumInto},
};

use super::Delta;

#[derive(Debug, Clone)]
pub struct Track<T> {
    event: T,
    pub track: u32,
}

impl<T> Track<T> {
    pub fn new(event: T, track: u32) -> Self {
        Self { event, track }
    }

    pub fn inner_event(self) -> T {
        self.event
    }
}

impl<T: MIDIEvent> std::ops::Deref for Track<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<T: MIDIEvent> std::ops::DerefMut for Track<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl<T: MIDIEvent> SerializeEvent for Track<T> {
    fn serialize_event<W: std::io::Write>(&self, buf: &mut W) -> Result<usize, MIDIWriteError> {
        self.event.serialize_event(buf)
    }
}

impl<E: MIDIEvent> MIDIEvent for Track<E> {
    fn key(&self) -> Option<u8> {
        self.event.key()
    }

    fn key_mut(&mut self) -> Option<&mut u8> {
        self.event.key_mut()
    }

    fn channel(&self) -> Option<u8> {
        self.event.channel()
    }

    fn channel_mut(&mut self) -> Option<&mut u8> {
        self.event.channel_mut()
    }

    fn as_u32(&self) -> Option<u32> {
        self.event.as_u32()
    }
}

impl<E: MIDIEventEnum> MIDIEventEnum for Track<E> {
    fn as_event(&self) -> &crate::events::Event {
        self.event.as_event()
    }

    fn as_event_mut(&mut self) -> &mut crate::events::Event {
        self.event.as_event_mut()
    }
}

impl<E: BatchTempo> BatchTempo for Track<E> {
    fn inner_tempo(&self) -> Option<u32> {
        self.event.inner_tempo()
    }

    fn without_tempo(self) -> Option<Self> {
        let track = self.track;
        self.event
            .without_tempo()
            .map(|event| Self::new(event, track))
    }
}

pub fn into_track_events<D: MIDINum, E, Err>(
    iter: impl Iterator<Item = Result<Delta<D, E>, Err>>,
    track: u32,
) -> impl Iterator<Item = Result<Delta<D, Track<E>>, Err>> {
    iter.map(move |e| e.map(|e| Delta::new(e.delta, Track::new(e.event, track))))
}
