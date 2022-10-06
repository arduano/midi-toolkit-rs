use crate::events::encode_var_length_value;
use crate::io::MIDIWriteError;
use crate::sequence::event::Delta;

use super::event::Event;
use super::{
    CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent, MIDINum, MIDINumInto, PlaybackEvent,
    SerializeEvent,
};
use derive::{MIDIEvent, NewEvent};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MIDIColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TextEventKind {
    TextEvent = 1,
    CopyrightNotice = 2,
    TrackName = 3,
    InstrumentName = 4,
    Lyric = 5,
    Marker = 6,
    CuePoint = 7,
    ProgramName = 8,
    DeviceName = 9,
    Undefined = 10,
    MetaEvent = 0x7F,
}

impl TextEventKind {
    pub fn from_val(val: u8) -> TextEventKind {
        match val {
            1 => TextEventKind::TextEvent,
            2 => TextEventKind::CopyrightNotice,
            3 => TextEventKind::TrackName,
            4 => TextEventKind::InstrumentName,
            5 => TextEventKind::Lyric,
            6 => TextEventKind::Marker,
            7 => TextEventKind::CuePoint,
            8 => TextEventKind::ProgramName,
            9 => TextEventKind::DeviceName,
            10 => TextEventKind::Undefined,
            0x7F => TextEventKind::MetaEvent,
            _ => panic!("Unrecognized text event kind received: {}", val),
        }
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct NoteOnEvent {
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
    pub velocity: u8,
}

impl SerializeEvent for NoteOnEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0x90 | self.channel, self.key, self.velocity];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for NoteOnEvent {
    fn as_u32(&self) -> u32 {
        (0x90 | self.channel as u32) | (self.key as u32) << 8 | (self.velocity as u32) << 16
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct NoteOffEvent {
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
}

impl SerializeEvent for NoteOffEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0x80 | self.channel, self.key, 0];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for NoteOffEvent {
    fn as_u32(&self) -> u32 {
        (0x80 | self.channel as u32) | (self.key as u32) << 8
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct PolyphonicKeyPressureEvent {
    #[channel]
    pub channel: u8,
    #[key]
    pub key: u8,
    pub velocity: u8,
}

impl SerializeEvent for PolyphonicKeyPressureEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xA0 | self.channel, self.key, self.velocity];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for PolyphonicKeyPressureEvent {
    fn as_u32(&self) -> u32 {
        (0xA0 | self.channel as u32) | (self.key as u32) << 8 | (self.velocity as u32) << 16
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct ControlChangeEvent {
    #[channel]
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

impl SerializeEvent for ControlChangeEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xB0 | self.channel, self.controller, self.value];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for ControlChangeEvent {
    fn as_u32(&self) -> u32 {
        (0xB0 | self.channel as u32) | (self.controller as u32) << 8 | (self.value as u32) << 16
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct ProgramChangeEvent {
    #[channel]
    pub channel: u8,
    pub program: u8,
}

impl SerializeEvent for ProgramChangeEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xC0 | self.channel, self.program];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for ProgramChangeEvent {
    fn as_u32(&self) -> u32 {
        (0xC0 | self.channel as u32) | (self.program as u32) << 8
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct ChannelPressureEvent {
    #[channel]
    pub channel: u8,
    pub pressure: u8,
}

impl SerializeEvent for ChannelPressureEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xD0 | self.channel, self.pressure];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for ChannelPressureEvent {
    fn as_u32(&self) -> u32 {
        (0xD0 | self.channel as u32) | (self.pressure as u32) << 8
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
#[playback]
pub struct PitchWheelChangeEvent {
    #[channel]
    pub channel: u8,
    pub pitch: i16,
}

impl SerializeEvent for PitchWheelChangeEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let value = self.pitch + 8192;
        let event = [
            0xE0 | self.channel,
            (value & 0x7F) as u8,
            ((value >> 7) & 0x7F) as u8,
        ];
        Ok(buf.write(&event)?)
    }
}

impl PlaybackEvent for PitchWheelChangeEvent {
    fn as_u32(&self) -> u32 {
        let value = self.pitch + 8192;
        let val1 = value & 0x7F;
        let val2 = (value >> 7) & 0x7F;
        (0xE0 | self.channel as u32) | (val1 as u32) << 8 | (val2 as u32) << 16
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct SystemExclusiveMessageEvent {
    pub data: Vec<u8>,
}

impl SerializeEvent for SystemExclusiveMessageEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let mut vec = Vec::new();
        vec.reserve(self.data.len() + 2);
        vec.push(0xF0u8);
        for v in self.data.iter() {
            vec.push(*v);
        }
        vec.push(0xF7u8);
        Ok(buf.write(&vec)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct UndefinedEvent {
    pub event: u8,
}

impl SerializeEvent for UndefinedEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [self.event];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct SongPositionPointerEvent {
    pub position: u16,
}

impl SerializeEvent for SongPositionPointerEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [
            0xF2,
            (self.position & 0x7F) as u8,
            ((self.position >> 7) & 0x7F) as u8,
        ];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct SongSelectEvent {
    pub song: u8,
}

impl SerializeEvent for SongSelectEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xF3, self.song];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TuneRequestEvent {}

impl SerializeEvent for TuneRequestEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xF6];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct EndOfExclusiveEvent {}

impl SerializeEvent for EndOfExclusiveEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xF7];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TrackStartEvent {}

impl SerializeEvent for TrackStartEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xFF, 0x00, 0x02];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TextEvent {
    pub kind: TextEventKind,
    pub bytes: Vec<u8>,
}

impl SerializeEvent for TextEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let mut vec = Vec::new();
        vec.reserve(self.bytes.len() + 2);
        vec.push(0xFF);
        vec.push(self.kind as u8);
        vec.append(&mut encode_var_length_value(self.bytes.len() as u64));
        for v in self.bytes.iter() {
            vec.push(*v);
        }
        Ok(buf.write(&vec)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct UnknownMetaEvent {
    pub kind: u8,
    pub bytes: Vec<u8>,
}

impl SerializeEvent for UnknownMetaEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let mut vec = Vec::new();
        vec.reserve(self.bytes.len() + 2);
        vec.push(0xFF);
        vec.push(self.kind as u8);
        vec.append(&mut encode_var_length_value(self.bytes.len() as u64));
        for v in self.bytes.iter() {
            vec.push(*v);
        }
        Ok(buf.write(&vec)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct ColorEvent {
    pub channel: u8,
    pub col: MIDIColor,
    pub col2: Option<MIDIColor>,
}

impl SerializeEvent for ColorEvent {
    fn serialize_event<T: std::io::Write>(&self, _buf: &mut T) -> Result<usize, MIDIWriteError> {
        todo!();
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct ChannelPrefixEvent {
    #[channel]
    pub channel: u8,
}

impl SerializeEvent for ChannelPrefixEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xFF, 0x20, 0x01, self.channel];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct MIDIPortEvent {
    #[channel]
    pub channel: u8,
}

impl SerializeEvent for MIDIPortEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xFF, 0x21, 0x01, self.channel];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TempoEvent {
    pub tempo: u32,
}

impl SerializeEvent for TempoEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [
            0xFF,
            0x51,
            0x03,
            ((self.tempo >> 16) & 0xFF) as u8,
            ((self.tempo >> 8) & 0xFF) as u8,
            (self.tempo & 0xFF) as u8,
        ];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct SMPTEOffsetEvent {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub fractional_frames: u8,
}

impl SerializeEvent for SMPTEOffsetEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [
            0xFF,
            0x54,
            0x05,
            self.hours,
            self.minutes,
            self.seconds,
            self.frames,
            self.fractional_frames,
        ];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TimeSignatureEvent {
    pub numerator: u8,
    pub denominator: u8,
    pub ticks_per_click: u8,
    pub bb: u8,
}

impl SerializeEvent for TimeSignatureEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [
            0xFF,
            0x58,
            0x04,
            self.numerator,
            self.denominator,
            self.ticks_per_click,
            self.bb,
        ];
        Ok(buf.write(&event)?)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct KeySignatureEvent {
    pub sf: u8,
    pub mi: u8,
}

impl SerializeEvent for KeySignatureEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xFF, 0x59, 0x02, self.sf, self.mi];
        Ok(buf.write(&event)?)
    }
}
