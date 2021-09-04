use super::events::*;
use super::{CastEventDelta, ChannelEvent, KeyEvent, PlaybackEvent, MIDIEvent, SerializeEvent};
use crate::io::MIDIWriteError;
use crate::num::{MIDINum, MIDINumInto};
use derive::EventImpl;
use std::io::Write;

#[derive(EventImpl, Debug, PartialEq)]
pub enum Event<D: MIDINum> {
    #[key]
    #[channel]
    #[playback]
    NoteOn(NoteOnEvent<D>),
    #[key]
    #[channel]
    #[playback]
    NoteOff(NoteOffEvent<D>),
    #[key]
    #[channel]
    #[playback]
    PolyphonicKeyPressure(Box<PolyphonicKeyPressureEvent<D>>),
    #[channel]
    #[playback]
    ControlChange(Box<ControlChangeEvent<D>>),
    #[channel]
    #[playback]
    ProgramChange(Box<ProgramChangeEvent<D>>),
    #[channel]
    #[playback]
    ChannelPressure(Box<ChannelPressureEvent<D>>),
    #[channel]
    #[playback]
    PitchWheelChange(Box<PitchWheelChangeEvent<D>>),
    SystemExclusiveMessage(Box<SystemExclusiveMessageEvent<D>>),
    Undefined(Box<UndefinedEvent<D>>),
    SongPositionPointer(Box<SongPositionPointerEvent<D>>),
    SongSelect(Box<SongSelectEvent<D>>),
    TuneRequest(Box<TuneRequestEvent<D>>),
    EndOfExclusive(Box<EndOfExclusiveEvent<D>>),
    TrackStart(Box<TrackStartEvent<D>>),
    Text(Box<TextEvent<D>>),
    UnknownMeta(Box<UnknownMetaEvent<D>>),
    Color(Box<ColorEvent<D>>),
    #[channel]
    ChannelPrefix(Box<ChannelPrefixEvent<D>>),
    #[channel]
    MIDIPort(Box<MIDIPortEvent<D>>),
    Tempo(Box<TempoEvent<D>>),
    SMPTEOffset(Box<SMPTEOffsetEvent<D>>),
    TimeSignature(Box<TimeSignatureEvent<D>>),
    KeySignature(Box<KeySignatureEvent<D>>),
}

impl<D: MIDINum> Event<D> {}
