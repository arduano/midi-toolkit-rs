use super::event::Event;
use super::{CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent, MIDINum, MIDINumInto};
use derive::{CastEventDelta, MIDIEvent, NewEvent};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MIDIColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct NoteOnEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
    pub velocity: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct NoteOffEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct PolyphonicKeyPressureEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
    pub velocity: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ControlChangeEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ProgramChangeEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub program: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ChannelPressureEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub pressure: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct PitchWheelChangeEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub pitch: i16,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ChannelModeMessageEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub cc: u8,
    pub vv: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SystemExclusiveMessageEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub data: Box<[u8]>,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct UndefinedEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub event: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SongPositionPointerEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub position: u16,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SongSelectEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub song: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TuneRequestEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct EndOfExclusiveEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct MajorMidiMessageEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub data: Box<[u8]>,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TrackStartEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TextEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub kind: u8,
    pub bytes: Box<[u8]>,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ColorEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub channel: u8,
    pub col: MIDIColor,
    pub col2: Option<MIDIColor>,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ChannelPrefixEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub channel: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct MIDIPortEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub channel: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TempoEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub tempo: u32,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SMPTEOffsetEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub fractional_frames: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TimeSignatureEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub numerator: u8,
    pub denominator: u8,
    pub ticks_per_click: u8,
    pub bb: u8,
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct KeySignatureEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub sf: u8,
    pub mi: u8,
}
