use crate::events::encode_var_length_value;
use crate::io::MIDIWriteError;

use super::event::Event;
use super::{
    CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent, MIDINum, MIDINumInto, SerializeEvent,
};
use derive::{CastEventDelta, MIDIEvent, NewEvent};

macro_rules! midi_error {
    ($val:expr) => {
        match $val {
            Ok(_) => Ok(()),
            Err(e) => Err(MIDIWriteError::FilesystemError(e)),
        }
    };
}

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

impl<D: MIDINum> SerializeEvent for NoteOnEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0x90 | self.channel, self.key, self.velocity];
        midi_error!(buf.write(&event))
    }
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

impl<D: MIDINum> SerializeEvent for NoteOffEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0x80 | self.channel, self.key, 0];
        midi_error!(buf.write(&event))
    }
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

impl<D: MIDINum> SerializeEvent for PolyphonicKeyPressureEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xA0 | self.channel, self.key, self.velocity];
        midi_error!(buf.write(&event))
    }
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

impl<D: MIDINum> SerializeEvent for ControlChangeEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xB0 | self.channel, self.controller, self.value];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ProgramChangeEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub program: u8,
}

impl<D: MIDINum> SerializeEvent for ProgramChangeEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xC0 | self.channel, self.program];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ChannelPressureEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub pressure: u8,
}

impl<D: MIDINum> SerializeEvent for ChannelPressureEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xD0 | self.channel, self.pressure];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct PitchWheelChangeEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
    pub pitch: i16,
}

impl<D: MIDINum> SerializeEvent for PitchWheelChangeEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let value = self.pitch + 8192;
        let event = [
            0xE0 | self.channel,
            (value & 0x7F) as u8,
            ((value >> 7) & 0x7F) as u8,
        ];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SystemExclusiveMessageEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub data: Vec<u8>,
}

impl<D: MIDINum> SerializeEvent for SystemExclusiveMessageEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let mut vec = Vec::new();
        vec.reserve(self.data.len() + 2);
        vec.push(0xF0u8);
        for v in self.data.iter() {
            vec.push(*v);
        }
        vec.push(0xF7u8);
        midi_error!(buf.write(&vec))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct UndefinedEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub event: u8,
}

impl<D: MIDINum> SerializeEvent for UndefinedEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [self.event];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SongPositionPointerEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub position: u16,
}

impl<D: MIDINum> SerializeEvent for SongPositionPointerEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [
            0xF2,
            (self.position & 0x7F) as u8,
            ((self.position >> 7) & 0x7F) as u8,
        ];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct SongSelectEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub song: u8,
}

impl<D: MIDINum> SerializeEvent for SongSelectEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xF3, self.song];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TuneRequestEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
}

impl<D: MIDINum> SerializeEvent for TuneRequestEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xF6];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct EndOfExclusiveEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
}

impl<D: MIDINum> SerializeEvent for EndOfExclusiveEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xF7];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TrackStartEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
}

impl<D: MIDINum> SerializeEvent for TrackStartEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xFF, 0x00, 0x02];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TextEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub kind: TextEventKind,
    pub bytes: Vec<u8>,
}

impl<D: MIDINum> SerializeEvent for TextEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let mut vec = Vec::new();
        vec.reserve(self.bytes.len() + 2);
        vec.push(0xFF);
        vec.push(self.kind as u8);
        vec.append(&mut encode_var_length_value(self.bytes.len() as u64));
        for v in self.bytes.iter() {
            vec.push(*v);
        }
        midi_error!(buf.write(&vec))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct UnknownMetaEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub kind: u8,
    pub bytes: Vec<u8>,
}

impl<D: MIDINum> SerializeEvent for UnknownMetaEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let mut vec = Vec::new();
        vec.reserve(self.bytes.len() + 2);
        vec.push(0xFF);
        vec.push(self.kind as u8);
        vec.append(&mut encode_var_length_value(self.bytes.len() as u64));
        for v in self.bytes.iter() {
            vec.push(*v);
        }
        midi_error!(buf.write(&vec))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ColorEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub channel: u8,
    pub col: MIDIColor,
    pub col2: Option<MIDIColor>,
}

impl<D: MIDINum> SerializeEvent for ColorEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, _buf: &mut T) -> Result<(), MIDIWriteError> {
        todo!();
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct ChannelPrefixEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
}

impl<D: MIDINum> SerializeEvent for ChannelPrefixEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xFF, 0x20, 0x01, self.channel];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct MIDIPortEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    #[channel]
    pub channel: u8,
}

impl<D: MIDINum> SerializeEvent for MIDIPortEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xFF, 0x21, 0x01, self.channel];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct TempoEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub tempo: u32,
}

impl<D: MIDINum> SerializeEvent for TempoEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [
            0xFF,
            0x51,
            0x03,
            ((self.tempo >> 16) & 0xFF) as u8,
            ((self.tempo >> 8) & 0xFF) as u8,
            (self.tempo & 0xFF) as u8,
        ];
        midi_error!(buf.write(&event))
    }
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

impl<D: MIDINum> SerializeEvent for SMPTEOffsetEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
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
        midi_error!(buf.write(&event))
    }
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

impl<D: MIDINum> SerializeEvent for TimeSignatureEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [
            0xFF,
            0x58,
            0x04,
            self.numerator,
            self.denominator,
            self.ticks_per_click,
            self.bb,
        ];
        midi_error!(buf.write(&event))
    }
}

#[derive(Debug, MIDIEvent, CastEventDelta, Clone, NewEvent, PartialEq)]
pub struct KeySignatureEvent<D: MIDINum> {
    #[delta]
    pub delta: D,
    pub sf: u8,
    pub mi: u8,
}

impl<D: MIDINum> SerializeEvent for KeySignatureEvent<D> {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<(), MIDIWriteError> {
        let event = [0xFF, 0x59, 0x02, self.sf, self.mi];
        midi_error!(buf.write(&event))
    }
}
