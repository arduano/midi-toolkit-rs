use super::{events::*, BatchTempo};
use super::{MIDIEvent, SerializeEvent};
use crate::io::MIDIWriteError;

use derive::EventImpl;
use std::io::Write;

#[derive(EventImpl, Debug, PartialEq)]
pub enum Event {
    #[key]
    #[channel]
    #[playback]
    NoteOn(NoteOnEvent),
    #[key]
    #[channel]
    #[playback]
    NoteOff(NoteOffEvent),
    #[key]
    #[channel]
    #[playback]
    PolyphonicKeyPressure(Box<PolyphonicKeyPressureEvent>),
    #[channel]
    #[playback]
    ControlChange(Box<ControlChangeEvent>),
    #[channel]
    #[playback]
    ProgramChange(Box<ProgramChangeEvent>),
    #[channel]
    #[playback]
    ChannelPressure(Box<ChannelPressureEvent>),
    #[channel]
    #[playback]
    PitchWheelChange(Box<PitchWheelChangeEvent>),
    SystemExclusiveMessage(Box<SystemExclusiveMessageEvent>),
    Undefined(Box<UndefinedEvent>),
    SongPositionPointer(Box<SongPositionPointerEvent>),
    SongSelect(Box<SongSelectEvent>),
    TuneRequest(Box<TuneRequestEvent>),
    EndOfExclusive(Box<EndOfExclusiveEvent>),
    TrackStart(Box<TrackStartEvent>),
    Text(Box<TextEvent>),
    UnknownMeta(Box<UnknownMetaEvent>),
    Color(Box<ColorEvent>),
    #[channel]
    ChannelPrefix(Box<ChannelPrefixEvent>),
    #[channel]
    MIDIPort(Box<MIDIPortEvent>),
    Tempo(Box<TempoEvent>),
    SMPTEOffset(Box<SMPTEOffsetEvent>),
    TimeSignature(Box<TimeSignatureEvent>),
    KeySignature(Box<KeySignatureEvent>),
}

impl Event {}

impl BatchTempo for Event {
    fn inner_tempo(&self) -> Option<u32> {
        match self {
            Event::Tempo(e) => Some(e.tempo),
            _ => None,
        }
    }

    fn without_tempo(self) -> Option<Self> {
        match self {
            Event::Tempo(_) => None,
            _ => Some(self),
        }
    }
}
