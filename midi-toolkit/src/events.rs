use std::io::Write;

use crate::{
    io::MIDIWriteError,
    num::{MIDINum, MIDINumInto},
};
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

pub trait MIDIEvent<T: MIDINum>: SerializeEvent + std::fmt::Debug {
    fn delta(&self) -> T;
    fn delta_mut(&mut self) -> &mut T;

    fn key(&self) -> Option<u8>;
    fn key_mut(&mut self) -> Option<&mut u8>;
    fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>>;
    fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>>;

    fn channel(&self) -> Option<u8>;
    fn channel_mut(&mut self) -> Option<&mut u8>;
    fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>>;
    fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>>;

    fn as_u32(&self) -> Option<u32>;
    fn as_playback_event<'a>(&'a self) -> Option<Box<&'a dyn PlaybackEvent>>;

    #[inline(always)]
    fn set_delta(&mut self, delta: T) {
        *self.delta_mut() = delta;
    }
}

pub trait MIDIEventEnum<T: MIDINum>: MIDIEvent<T> {
    fn as_event(&self) -> &Event<T>;
    fn as_event_mut(&mut self) -> &mut Event<T>;
}

impl<E: MIDIEvent<u64>> SerializeEventWithDelta for E {
    fn serialize_delta<T: Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let delta = self.delta();
        let vec = encode_var_length_value(delta);
        Ok(buf.write(&vec)?)
    }
}

impl<T: MIDINum> MIDIEventEnum<T> for Event<T> {
    fn as_event(&self) -> &Event<T> {
        self
    }

    fn as_event_mut(&mut self) -> &mut Event<T> {
        self
    }
}

pub trait CastEventDelta<DT: MIDINum, Ev: MIDIEvent<DT>>: Clone {
    /// Casts the delta time of MIDIEvent to a different type
    ///
    /// By default, supports: i32, i64, u32, u64, f32, f64
    /// ## Example
    /// ```
    ///use midi_toolkit::events::{CastEventDelta, Event, MIDIEvent};
    ///
    ///let note_on_i32 = Event::new_note_on_event(10i32, 0, 64, 127);
    ///let note_on_u64 = Event::new_note_on_event(10u64, 0, 64, 127);
    ///
    ///let note_on_f32: Event<f32> = note_on_i32.cast_delta();
    ///let note_on_f64: Event<f64> = note_on_i32.cast_delta();
    ///let note_on_u32: Event<u32> = note_on_u64.cast_delta();
    ///let note_on_i64: Event<i64> = note_on_u64.cast_delta();
    ///
    ///assert_eq!(note_on_f32.delta(), 10f32);
    ///assert_eq!(note_on_f64.delta(), 10f64);
    ///assert_eq!(note_on_u32.delta(), 10u32);
    ///assert_eq!(note_on_i64.delta(), 10i64);
    /// ```
    fn cast_delta(&self) -> Ev;
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
        events::{CastEventDelta, Event, MIDIEvent},
        io::FullRamTrackReader,
    };

    #[test]
    fn test_cast_delta() {
        let note_on_i32 = Event::new_note_on_event(10i32, 0, 64, 127);
        let note_on_u64 = Event::new_note_on_event(10u64, 0, 64, 127);

        let note_on_f32: Event<f32> = note_on_i32.cast_delta();
        let note_on_f64: Event<f64> = note_on_i32.cast_delta();
        let note_on_u32: Event<u32> = note_on_u64.cast_delta();
        let note_on_i64: Event<i64> = note_on_u64.cast_delta();

        assert_eq!(note_on_f32.delta(), 10f32);
        assert_eq!(note_on_f64.delta(), 10f64);
        assert_eq!(note_on_u32.delta(), 10u32);
        assert_eq!(note_on_i64.delta(), 10i64);
    }

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

    fn parse_from_vec(mut vec: Vec<u8>) -> Event<u64> {
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
