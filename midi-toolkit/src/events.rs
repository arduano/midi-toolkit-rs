use std::io::Write;

use crate::{io::MIDIWriteError, num::MIDINum};
pub use event::Event;
pub use events::*;

mod event;
mod events;

pub fn encode_var_length_value(mut val: u64) -> Vec<u8> {
    let mut vec = Vec::new();
    let mut added = 0x00u8;
    loop {
        let v = (val & 0x7F) as u8 | added;
        vec.push(v);
        val = val >> 7;
        added = 0x80;
        if val == 0 {
            break;
        }
    }
    vec.reverse();
    vec
}

pub trait SerializeEvent {
    fn serialize_event<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError>;
}

pub trait SerializeEventWithDelta: SerializeEvent {
    fn serialize_delta<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError>;
    fn serialize_event_with_delta<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        Ok(self.serialize_delta(buf)? + self.serialize_event(buf)?)
    }
}

pub trait MIDIEvent: SerializeEvent + std::fmt::Debug {
    fn key(&self) -> Option<u8>;
    fn key_mut(&mut self) -> Option<&mut u8>;

    fn channel(&self) -> Option<u8>;
    fn channel_mut(&mut self) -> Option<&mut u8>;

    fn as_u32(&self) -> Option<u32>;
}

pub trait BatchTempo: Sized {
    fn inner_tempo(&self) -> Option<u32>;
    fn without_tempo(self) -> Option<Self>;
}

pub trait MIDIEventEnum: MIDIEvent {
    fn as_event(&self) -> &Event;
    fn as_event_mut(&mut self) -> &mut Event;
}

pub trait MIDIDelta<D: MIDINum> {
    fn delta(&self) -> D;
    fn delta_mut(&mut self) -> &mut D;

    #[inline(always)]
    fn set_delta(&mut self, delta: D) {
        *self.delta_mut() = delta;
    }
}

impl MIDIEventEnum for Event {
    fn as_event(&self) -> &Event {
        self
    }

    fn as_event_mut(&mut self) -> &mut Event {
        self
    }
}

pub trait CastEventDelta<DT: MIDINum> {
    type Output;

    fn cast_delta(self) -> Self::Output;
}

/// A trait that describes an event that is always connected to a channel
pub trait ChannelEvent {
    fn channel(&self) -> u8;
    fn channel_mut(&mut self) -> &mut u8;
}

/// A trait that describes an event that is always connected to a key
pub trait KeyEvent: ChannelEvent {
    fn key(&self) -> u8;
    fn key_mut(&mut self) -> &mut u8;
}

/// A trait that describes an event that is always serializable to u32 for playback
pub trait PlaybackEvent: ChannelEvent {
    fn as_u32(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use crate::{
        events::{Event, MIDIEvent},
        io::FullRamTrackReader,
        sequence::event::Delta,
    };

    fn make_example_playback_events() -> Vec<Vec<u8>> {
        vec![
            vec![0x82, 0x40, 0x00],
            vec![0x94, 0x40, 0x20],
            vec![0xA3, 0x40, 0x20],
            vec![0xA3, 0x40, 0x20],
            vec![0xBF, 0x32, 0x12],
            vec![0xC0, 0x14],
            vec![0xD5, 0x7F],
            vec![0xE7, 0x23, 0x68],
        ]
    }

    fn parse_from_vec(mut vec: Vec<u8>) -> Delta<u64, Event> {
        vec.insert(0, 64);
        let reader = FullRamTrackReader::new_from_vec(vec);
        let mut parser = crate::io::TrackParser::new(reader);
        parser.next().unwrap().unwrap()
    }

    #[test]
    fn end_to_end_parse_serialize() {
        let events = make_example_playback_events();
        for event_bytes in events.iter() {
            let event = parse_from_vec(event_bytes.clone());
            let serialized = event.as_u32().unwrap();
            let mut compressed: u32 = 0x00;
            let mut offset: u32 = 0;
            for v in event_bytes.iter() {
                compressed = compressed | ((*v as u32) << offset);
                offset += 8;
            }
            assert_eq!(serialized, compressed);
        }
    }
}
