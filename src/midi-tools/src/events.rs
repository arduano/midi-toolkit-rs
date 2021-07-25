pub use derive::{MIDIEvent, CastEventDelta};

mod delta;
pub use delta::{DeltaNum, DeltaNumInto};

pub trait MIDIEvent<T: DeltaNum> {
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

pub trait CastEventDelta<DT: DeltaNum, Ev: MIDIEvent<DT>> {
    fn clone(&self) -> Self;
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

#[derive(Debug, MIDIEvent, CastEventDelta)]
pub struct NoteOnEvent<D: DeltaNum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
    pub velocity: u8,
}

#[derive(Debug, MIDIEvent, Clone)]
pub struct NoteOffEvent<D: DeltaNum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
}

enum Event<D: DeltaNum> {
    NoteOn(Box<NoteOnEvent<D>>),
    NoteOff(Box<NoteOffEvent<D>>),
}
