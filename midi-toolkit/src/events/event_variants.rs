use crate::events::encode_var_length_value;
use crate::io::MIDIWriteError;
use crate::sequence::event::Delta;
use std::io::{Error, ErrorKind, Write};

use super::event::Event;
use super::{ChannelEvent, KeyEvent, MIDIEvent, MIDINum, PlaybackEvent, SerializeEvent};
use derive::{MIDIEvent, NewEvent};

fn write_all_len<T: Write>(buf: &mut T, bytes: &[u8]) -> Result<usize, MIDIWriteError> {
    buf.write_all(bytes)?;
    Ok(bytes.len())
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
            _ => TextEventKind::Undefined,
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        let mut vec = Vec::with_capacity(self.data.len() + 5);
        vec.push(0xF0u8);
        vec.extend(encode_var_length_value(self.data.len() as u64));
        vec.extend(self.data.iter().copied());
        write_all_len(buf, &vec)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct UndefinedEvent {
    pub event: u8,
}

impl SerializeEvent for UndefinedEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [self.event];
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct SongSelectEvent {
    pub song: u8,
}

impl SerializeEvent for SongSelectEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xF3, self.song];
        write_all_len(buf, &event)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TuneRequestEvent {}

impl SerializeEvent for TuneRequestEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xF6];
        write_all_len(buf, &event)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct EndOfExclusiveEvent {}

impl SerializeEvent for EndOfExclusiveEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xF7];
        write_all_len(buf, &event)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TrackStartEvent {}

impl SerializeEvent for TrackStartEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let event = [0xFF, 0x00, 0x02, 0x00, 0x00];
        write_all_len(buf, &event)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct TextEvent {
    pub kind: TextEventKind,
    pub bytes: Vec<u8>,
}

impl SerializeEvent for TextEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let mut vec = Vec::with_capacity(self.bytes.len() + 2);
        vec.push(0xFF);
        vec.push(self.kind as u8);
        vec.append(&mut encode_var_length_value(self.bytes.len() as u64));
        for v in self.bytes.iter() {
            vec.push(*v);
        }
        write_all_len(buf, &vec)
    }
}

#[derive(Debug, MIDIEvent, Clone, NewEvent, PartialEq)]
pub struct UnknownMetaEvent {
    pub kind: u8,
    pub bytes: Vec<u8>,
}

impl SerializeEvent for UnknownMetaEvent {
    fn serialize_event<T: std::io::Write>(&self, buf: &mut T) -> Result<usize, MIDIWriteError> {
        let mut vec = Vec::with_capacity(self.bytes.len() + 2);
        vec.push(0xFF);
        vec.push(self.kind);
        vec.append(&mut encode_var_length_value(self.bytes.len() as u64));
        for v in self.bytes.iter() {
            vec.push(*v);
        }
        write_all_len(buf, &vec)
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
        Err(MIDIWriteError::FilesystemError(Error::new(
            ErrorKind::Unsupported,
            "ColorEvent serialization is not supported",
        )))
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
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
        write_all_len(buf, &event)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ColorEvent, MIDIColor, SerializeEvent, SystemExclusiveMessageEvent, TrackStartEvent,
    };
    use crate::io::MIDIWriteError;
    use std::io::{ErrorKind, Write};

    struct PartialWriter {
        bytes: Vec<u8>,
        max_write: usize,
    }

    impl PartialWriter {
        fn new(max_write: usize) -> Self {
            Self {
                bytes: Vec::new(),
                max_write,
            }
        }
    }

    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let written = self.max_write.min(buf.len());
            self.bytes.extend_from_slice(&buf[..written]);
            Ok(written)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn track_start_serializes_with_two_payload_bytes() {
        let mut buf = Vec::new();
        TrackStartEvent {}
            .serialize_event(&mut buf)
            .expect("track start should serialize");

        assert_eq!(buf, vec![0xFF, 0x00, 0x02, 0x00, 0x00]);
    }

    #[test]
    fn color_event_returns_typed_unsupported_error() {
        let mut buf = Vec::new();
        let err = ColorEvent {
            channel: 0,
            col: MIDIColor {
                r: 1,
                g: 2,
                b: 3,
                a: 4,
            },
            col2: None,
        }
        .serialize_event(&mut buf)
        .expect_err("color serialization should not panic");

        match err {
            MIDIWriteError::FilesystemError(err) => assert_eq!(err.kind(), ErrorKind::Unsupported),
            other => panic!("unexpected write error variant: {:?}", other),
        }
    }

    #[test]
    fn sysex_serializes_as_smf_event_without_implicit_terminator() {
        let mut buf = Vec::new();
        SystemExclusiveMessageEvent {
            data: vec![0x12, 0xF7, 0x34],
        }
        .serialize_event(&mut buf)
        .expect("sysex should serialize");

        assert_eq!(buf, vec![0xF0, 0x03, 0x12, 0xF7, 0x34]);
    }

    #[test]
    fn serializers_handle_partial_writes() {
        let mut buf = PartialWriter::new(1);

        let note = SystemExclusiveMessageEvent {
            data: vec![0x41, 0x42, 0x43],
        };

        note.serialize_event(&mut buf)
            .expect("write_all should handle partial writes");

        assert_eq!(buf.bytes, vec![0xF0, 0x03, 0x41, 0x42, 0x43]);
    }
}
