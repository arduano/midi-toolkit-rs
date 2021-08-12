mod event;
mod events;
use std::io::Write;

use crate::{
    io::MIDIWriteError,
    num::{MIDINum, MIDINumInto},
};
pub use event::Event;
pub use events::*;

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

macro_rules! midi_error {
    ($val:expr) => {
        match $val {
            Ok(_) => Ok(()),
            Err(e) => Err(MIDIWriteError::FilesystemError(e)),
        }
    };
}

pub trait SerializeEvent {
    fn serialize_event<T: Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError>;
}

pub trait SerializeEventWithDelta: SerializeEvent {
    fn serialize_delta<T: Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError>;
    fn serialize_event_with_delta<T: Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        self.serialize_delta(buf)?;
        self.serialize_event(buf)?;
        Ok(())
    }
}

pub trait MIDIEvent<T: MIDINum>: SerializeEvent {
    fn delta(&self) -> T;
    fn delta_mut(&mut self) -> &mut T;
    fn key(&self) -> Option<u8>;
    fn key_mut(&mut self) -> Option<&mut u8>;
    fn channel(&self) -> Option<u8>;
    fn channel_mut(&mut self) -> Option<&mut u8>;
    fn as_key_event<'a>(&'a self) -> Option<Box<&'a dyn KeyEvent>>;
    fn as_key_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn KeyEvent>>;
    fn as_channel_event<'a>(&'a self) -> Option<Box<&'a dyn ChannelEvent>>;
    fn as_channel_event_mut<'a>(&'a mut self) -> Option<Box<&'a mut dyn ChannelEvent>>;

    #[inline(always)]
    fn set_delta(&mut self, delta: T) {
        *self.delta_mut() = delta;
    }
}

pub trait MIDIEventEnum<T: MIDINum>: MIDIEvent<T> {
    fn as_event(&self) -> &Event<T>;
    fn as_event_mut(&mut self) -> &Event<T>;
}

impl<E: MIDIEvent<u64>> SerializeEventWithDelta for E {
    fn serialize_delta<T: Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let delta = self.delta();
        let vec = encode_var_length_value(delta);
        midi_error!(buf.write(&vec))
    }
}

impl<T: MIDINum> MIDIEventEnum<T> for Event<T> {
    fn as_event(&self) -> &Event<T> {
        self
    }

    fn as_event_mut(&mut self) -> &Event<T> {
        self
    }
}

pub trait CastEventDelta<DT: MIDINum, Ev: MIDIEvent<DT>>: Clone {
    /// Casts the delta time of MIDIEvent to a different type
    ///
    /// By default, supports: i32, i64, u32, u64, f32, f64
    /// ## Example
    /// ```
    ///use midi_tools::events::{CastEventDelta, Event, MIDIEvent};
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

pub trait ChannelEvent {
    fn channel(&self) -> u8;
    fn channel_mut(&mut self) -> &mut u8;
}

pub trait KeyEvent: ChannelEvent {
    fn key(&self) -> u8;
    fn key_mut(&mut self) -> &mut u8;
}
#[cfg(test)]
mod tests {
    use crate::events::{CastEventDelta, Event, MIDIEvent};

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
}
