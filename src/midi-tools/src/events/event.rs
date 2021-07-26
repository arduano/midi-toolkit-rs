use super::events::*;
use super::{ChannelEvent, CastEventDelta, MIDINum, MIDINumInto, KeyEvent, MIDIEvent};
use derive::EventImpl;

#[derive(EventImpl)]
pub enum Event<D: MIDINum> {
    #[key]
    #[channel]
    NoteOn(NoteOnEvent<D>),
    #[key]
    #[channel]
    NoteOff(NoteOffEvent<D>),
    #[channel]
    ControlChange(Box<ControlChangeEvent<D>>),
}

impl<D: MIDINum> Event<D> {
    
}