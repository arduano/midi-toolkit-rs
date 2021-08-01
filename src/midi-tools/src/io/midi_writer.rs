use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, copy, Cursor, Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

use crate::events::SerializeEventWithDelta;

use super::errors::MIDIWriteError;

pub trait WriteSeek: Write + Seek {}
impl WriteSeek for File {}
impl WriteSeek for Cursor<Vec<u8>> {}

pub struct QueuedOutput {
    write: Box<dyn Read>,
    length: u32,
}

struct TrackStatus {
    opened_tracks: HashSet<i32>,
    written_tracks: HashSet<i32>,
    next_init_track: i32,
    next_write_track: i32,
    queued_writes: HashMap<i32, QueuedOutput>,
}

pub struct MIDIWriter {
    output: Option<Mutex<Box<dyn WriteSeek>>>,
    tracks: Mutex<TrackStatus>,
}

pub struct TrackWriter<'a> {
    midi_writer: &'a MIDIWriter,
    track_id: i32,
    writer: Option<Cursor<Vec<u8>>>,
}

fn encode_u16(val: u16) -> [u8; 2] {
    let mut bytes = [0 as u8; 2];
    bytes[0] = ((val >> 8) & 0xff) as u8;
    bytes[1] = (val & 0xff) as u8;
    bytes
}

fn encode_u32(val: u32) -> [u8; 4] {
    let mut bytes = [0 as u8; 4];
    bytes[0] = ((val >> 24) & 0xff) as u8;
    bytes[1] = ((val >> 16) & 0xff) as u8;
    bytes[2] = ((val >> 8) & 0xff) as u8;
    bytes[3] = (val & 0xff) as u8;
    bytes
}

fn flush_track(writer: &mut Box<dyn WriteSeek>, mut output: QueuedOutput) -> Result<(), io::Error> {
    writer.write("MTrk".as_bytes())?;
    writer.write(&encode_u32(output.length))?;
    copy(&mut output.write, writer)?;
    Ok(())
}

macro_rules! midi_error {
    ($val:expr) => {
        match $val {
            Ok(e) => Ok(e),
            Err(e) => Err(MIDIWriteError::FilesystemError(e)),
        }
    };
}

macro_rules! midi_error_discard {
    ($val:expr) => {
        match $val {
            Ok(_) => Ok(()),
            Err(e) => Err(MIDIWriteError::FilesystemError(e)),
        }
    };
}

impl MIDIWriter {
    pub fn new(filename: &str, ppq: u16) -> Result<MIDIWriter, MIDIWriteError> {
        let reader = midi_error!(File::create(filename))?;
        MIDIWriter::new_from_stram(Box::new(reader), ppq)
    }

    pub fn new_from_stram(
        mut output: Box<dyn WriteSeek>,
        ppq: u16,
    ) -> Result<MIDIWriter, MIDIWriteError> {
        midi_error!(output.seek(SeekFrom::Start(0)))?;
        midi_error!(output.write("MThd".as_bytes()))?;
        midi_error!(output.write(&encode_u32(6)))?;
        midi_error!(output.write(&encode_u16(1)))?;
        midi_error!(output.write(&encode_u16(0)))?;
        midi_error!(output.write(&encode_u16(ppq)))?;

        Ok(MIDIWriter {
            output: Some(Mutex::new(output)),
            tracks: Mutex::new(TrackStatus {
                opened_tracks: HashSet::new(),
                next_init_track: 0,
                next_write_track: 0,
                queued_writes: HashMap::new(),
                written_tracks: HashSet::new(),
            }),
        })
    }

    fn get_writer(&self) -> &Mutex<Box<dyn WriteSeek>> {
        self.output
            .as_ref()
            .expect("Can't get the writer of an ended MIDIWriter")
    }

    fn write_u16_at(&self, at: u64, val: u16) -> Result<(), io::Error> {
        let mut output = self.get_writer().lock().unwrap();
        let pos = output.stream_position()?;
        output.seek(SeekFrom::Start(at))?;
        output.write(&encode_u16(val))?;
        output.seek(SeekFrom::Start(pos))?;
        Ok(())
    }

    pub fn write_ppq(&self, ppq: u16) -> Result<(), MIDIWriteError> {
        midi_error!(self.write_u16_at(12, ppq))
    }

    pub fn write_format(&self, ppq: u16) -> Result<(), MIDIWriteError> {
        midi_error!(self.write_u16_at(8, ppq))
    }

    fn write_ntrks(&self, ppq: u16) -> Result<(), MIDIWriteError> {
        midi_error!(self.write_u16_at(10, ppq))
    }

    pub fn open_next_track<'a>(&'a self) -> TrackWriter<'a> {
        let track_id = {
            let mut tracks = self.tracks.lock().unwrap();
            let track_id = tracks.next_init_track;
            tracks.next_init_track += 1;
            track_id
        };
        self.open_track(track_id)
    }

    pub fn open_track<'a>(&'a self, track_id: i32) -> TrackWriter<'a> {
        self.add_opened_track(track_id);
        TrackWriter {
            midi_writer: &self,
            track_id,
            writer: Some(Cursor::new(Vec::new())),
        }
    }

    fn add_opened_track(&self, track_id: i32) {
        let mut tracks = self.tracks.lock().unwrap();
        if tracks.written_tracks.contains(&track_id) || !tracks.opened_tracks.insert(track_id) {
            panic!("Track with id {} has aready been opened before", track_id);
        }
    }

    pub fn end(&mut self) -> Result<(), MIDIWriteError> {
        let tracks = self.tracks.lock().unwrap();
        if tracks.opened_tracks.len() > 0 {
            let unwritten: Vec<&i32> = tracks.queued_writes.keys().collect();
            panic!("Not all tracks have been ended! Make sure you drop or call .end() on each track before ending the MIDIWriter\nMissing tracks {:?}", unwritten);
        }
        if tracks.queued_writes.len() > 0 {
            let max_track = tracks.queued_writes.keys().max().unwrap();
            let unwritten: Vec<i32> = (0..*max_track)
                .into_iter()
                .filter(|track_id| !tracks.written_tracks.contains(&track_id))
                .collect();
            panic!(
                "Not all tracks have been opened! Missing tracks {:?}",
                unwritten
            );
        }

        let track_count = tracks.written_tracks.len();
        self.write_ntrks(track_count.min(u16::MAX as usize) as u16)?;

        self.output.take();

        Ok(())
    }

    pub fn is_ended(&self) -> bool {
        self.output.is_some()
    }
}

impl<'a> TrackWriter<'a> {
    pub fn end(&mut self) -> Result<(), MIDIWriteError> {
        self.write_bytes(&[0x00, 0xFF, 0x2F, 0x00])?;

        let mut status = self.midi_writer.tracks.lock().unwrap();
        if !status.written_tracks.insert(self.track_id)
            || !status.opened_tracks.remove(&self.track_id)
        {
            panic!("Invalid MIDIWriter state, unknown error");
        }

        let mut writer = match self.writer.take() {
            Some(cursor) => cursor,
            None => panic!(".end() was called more than once on TrackWriter"),
        };

        let length = midi_error!(writer.stream_position())? as u32;
        midi_error!(writer.seek(SeekFrom::Start(0)))?;

        status.queued_writes.insert(
            self.track_id,
            QueuedOutput {
                write: Box::new(writer),
                length,
            },
        );

        if self.track_id == status.next_write_track {
            let mut writer = self.midi_writer.get_writer().lock().unwrap();
            loop {
                let next_write_track = status.next_write_track;
                match status.queued_writes.remove_entry(&next_write_track) {
                    None => break,
                    Some(output) => {
                        midi_error!(flush_track(&mut writer, output.1))?;
                        status.next_write_track += 1;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_ended(&self) -> bool {
        self.writer.is_some()
    }

    pub fn get_writer_mut<'w>(&'w mut self) -> &'w mut impl Write {
        self.writer
            .as_mut()
            .expect("Tried to write to TrackWriter after .end() was called")
    }

    pub fn write_event<T: SerializeEventWithDelta>(
        &mut self,
        event: T,
    ) -> Result<(), MIDIWriteError> {
        let writer = self.get_writer_mut();
        event.serialize_event_with_delta(writer)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), MIDIWriteError> {
        let writer = self.get_writer_mut();
        midi_error_discard!(writer.write(bytes))
    }
}

impl<'a> Drop for TrackWriter<'a> {
    fn drop(&mut self) {
        if self.is_ended() {
            match self.end() {
                Ok(()) => {}
                Err(e) => {
                    panic!("TrackWriter errored when being dropped with: {:?}\n\nIf you want to handle these errors in the future, manually call .end() before dropping", e);
                }
            }
        }
    }
}

impl<'a> Drop for MIDIWriter {
    fn drop(&mut self) {
        if self.is_ended() {
            match self.end() {
                Ok(()) => {}
                Err(e) => {
                    panic!("TrackWriter errored when being dropped with: {:?}\n\nIf you want to handle these errors in the future, manually call .end() before dropping", e);
                }
            }
        }
    }
}
