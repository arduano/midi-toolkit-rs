use std::marker::PhantomData;

use crate::{
    events::{CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent, PlaybackEvent, SerializeEvent},
    io::MIDIWriteError,
    num::{MIDINum, MIDINumInto},
};

#[derive(Debug, Clone)]
pub struct TrackEvent<D: MIDINum, T: MIDIEvent<D>> {
    event: T,
    pub track: u32,
    _delta: PhantomData<D>,
}

impl<D: MIDINum, T: MIDIEvent<D>> TrackEvent<D, T> {
    pub fn new(event: T, track: u32) -> Self {
        Self {
            event,
            track,
            _delta: PhantomData,
        }
    }

    pub fn into(self) -> T {
        self.event
    }
}

impl<D: MIDINum, T: MIDIEvent<D>> std::ops::Deref for TrackEvent<D, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<D: MIDINum, T: MIDIEvent<D>> std::ops::DerefMut for TrackEvent<D, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl<D: MIDINum, T: MIDIEvent<D>> SerializeEvent for TrackEvent<D, T> {
    fn serialize_event<W: std::io::Write>(&self, buf: &mut W) -> Result<usize, MIDIWriteError> {
        self.event.serialize_event(buf)
    }
}

impl<D: MIDINum, T: MIDIEvent<D>> MIDIEvent<D> for TrackEvent<D, T> {
    fn delta(&self) -> D {
        self.event.delta()
    }

    fn delta_mut(&mut self) -> &mut D {
        self.event.delta_mut()
    }

    fn key(&self) -> Option<u8> {
        self.event.key()
    }

    fn key_mut(&mut self) -> Option<&mut u8> {
        self.event.key_mut()
    }

    fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>> {
        self.event.as_key_event()
    }

    fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>> {
        self.event.as_key_event_mut()
    }

    fn channel(&self) -> Option<u8> {
        self.event.channel()
    }

    fn channel_mut(&mut self) -> Option<&mut u8> {
        self.event.channel_mut()
    }

    fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>> {
        self.event.as_channel_event()
    }

    fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>> {
        self.event.as_channel_event_mut()
    }

    fn as_u32(&self) -> Option<u32> {
        self.event.as_u32()
    }

    fn as_playback_event<'a>(&'a self) -> Option<Box<&'a dyn PlaybackEvent>> {
        self.event.as_playback_event()
    }
}

impl<TT, DT: MIDINum, ETT: MIDIEvent<TT> + CastEventDelta<DT, EDT> + Clone, EDT: MIDIEvent<DT>>
    CastEventDelta<DT, TrackEvent<DT, EDT>> for TrackEvent<TT, ETT>
where
    TT: MIDINum + MIDINumInto<DT>,
{
    #[inline(always)]
    fn cast_delta(&self) -> TrackEvent<DT, EDT> {
        TrackEvent::new(self.event.cast_delta(), self.track)
    }
}

pub fn into_track_events<D: MIDINum, E: MIDIEvent<D>, Err>(
    iter: impl Iterator<Item = Result<E, Err>>,
    track: u32,
) -> impl Iterator<Item = Result<TrackEvent<D, E>, Err>> {
    iter.map(move |e| e.map(|e| TrackEvent::new(e, track)))
}
