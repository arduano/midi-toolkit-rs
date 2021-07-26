use super::events::*;
use super::{CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent};
use crate::num::{MIDINum, MIDINumInto};
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

impl<D: MIDINum> Event<D> {}
