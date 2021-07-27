use super::events::*;
use super::{CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent};
use crate::num::{MIDINum, MIDINumInto};
use derive::EventImpl;

#[derive(EventImpl, Debug, PartialEq)]
pub enum Event<D: MIDINum> {
    #[key]
    #[channel]
    NoteOn(NoteOnEvent<D>),
    #[key]
    #[channel]
    NoteOff(NoteOffEvent<D>),
    #[key]
    #[channel]
    PolyphonicKeyPressure(Box<PolyphonicKeyPressureEvent<D>>),
    #[channel]
    ControlChange(Box<ControlChangeEvent<D>>),
    #[channel]
    ProgramChange(Box<ProgramChangeEvent<D>>),
    #[channel]
    ChannelPressure(Box<ChannelPressureEvent<D>>),
    #[channel]
    PitchWheelChange(Box<PitchWheelChangeEvent<D>>),
    #[channel]
    ChannelModeMessage(Box<ChannelModeMessageEvent<D>>),
    SystemExclusiveMessage(Box<SystemExclusiveMessageEvent<D>>),
    Undefined(Box<UndefinedEvent<D>>),
    SongPositionPointer(Box<SongPositionPointerEvent<D>>),
    SongSelect(Box<SongSelectEvent<D>>),
    TuneRequest(Box<TuneRequestEvent<D>>),
    EndOfExclusive(Box<EndOfExclusiveEvent<D>>),
    MajorMidiMessage(Box<MajorMidiMessageEvent<D>>),
    TrackStart(Box<TrackStartEvent<D>>),
    Text(Box<TextEvent<D>>),
    Color(Box<ColorEvent<D>>),
    ChannelPrefix(Box<ChannelPrefixEvent<D>>),
    MIDIPort(Box<MIDIPortEvent<D>>),
    Tempo(Box<TempoEvent<D>>),
    SMPTEOffset(Box<SMPTEOffsetEvent<D>>),
    TimeSignature(Box<TimeSignatureEvent<D>>),
    KeySignature(Box<KeySignatureEvent<D>>),
}

impl<D: MIDINum> Event<D> {}
