use std::{
    io::Write,
    ops::{Deref, DerefMut},
};

use crate::{
    events::{
        encode_var_length_value, BatchTempo, CastEventDelta, MIDIDelta, MIDIEvent, MIDIEventEnum,
        SerializeEvent, SerializeEventWithDelta,
    },
    io::MIDIWriteError,
    num::{MIDINum, MIDINumInto},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delta<D: MIDINum, E> {
    pub delta: D,
    pub event: E,
}

impl<D: MIDINum, E> MIDIDelta<D> for Delta<D, E> {
    #[inline(always)]
    fn delta(&self) -> D {
        self.delta
    }

    #[inline(always)]
    fn delta_mut(&mut self) -> &mut D {
        &mut self.delta
    }
}

impl<D: MIDINum + MIDINumInto<ND>, ND: MIDINum, E> CastEventDelta<ND> for Delta<D, E> {
    type Output = Delta<ND, E>;

    #[inline(always)]
    fn cast_delta(self) -> Self::Output {
        Delta {
            delta: self.delta.midi_num_into(),
            event: self.event,
        }
    }
}

impl<D: MIDINum, E> Delta<D, E> {
    #[inline(always)]
    pub fn new(delta: D, event: E) -> Self {
        Self { delta, event }
    }
}

impl<D: MIDINum, E> Deref for Delta<D, E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<D: MIDINum, E> DerefMut for Delta<D, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.event
    }
}

impl<D: MIDINum, E: MIDIEventEnum> MIDIEvent for Delta<D, E> {
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

impl<D: MIDINum, E: MIDIEventEnum> MIDIEventEnum for Delta<D, E> {
    #[inline(always)]
    fn as_event(&self) -> &crate::events::Event {
        self.event.as_event()
    }

    #[inline(always)]
    fn as_event_mut(&mut self) -> &mut crate::events::Event {
        self.event.as_event_mut()
    }
}

impl<D: MIDINum, E: BatchTempo> BatchTempo for Delta<D, E> {
    fn inner_tempo(&self) -> Option<u32> {
        self.event.inner_tempo()
    }

    fn without_tempo(self) -> Option<Self> {
        let delta = self.delta;
        self.event
            .without_tempo()
            .map(|event| Self::new(delta, event))
    }
}

impl<D: MIDINum, E: SerializeEvent> SerializeEvent for Delta<D, E> {
    fn serialize_event<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        Ok(self.event.serialize_event(buf)?)
    }
}

impl<E: SerializeEvent> SerializeEventWithDelta for Delta<u64, E> {
    fn serialize_delta<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let vec = encode_var_length_value(self.delta);
        buf.write_all(&vec)?;
        Ok(vec.len())
    }
}
