mod event;
mod events;
use crate::num::{MIDINum, MIDINumInto};
pub use event::Event;
pub use events::*;

pub trait MIDIEvent<T: MIDINum> {
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
