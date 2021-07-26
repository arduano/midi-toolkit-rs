use super::{CastEventDelta, ChannelEvent, MIDINum, MIDINumInto, KeyEvent, MIDIEvent};
use super::event::Event;
use derive::{MIDIEvent, CastEventDelta, NewEvent};

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent)]
pub struct NoteOnEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
    pub velocity: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent)]
pub struct NoteOffEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent)]
pub struct ControlChangeEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}
