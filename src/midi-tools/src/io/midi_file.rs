use std::{
    fs::File,
    io::{Read, Seek},
};

use crate::events::Event;
use std::fmt::Debug;

use super::{
    errors::{MIDILoadError, MIDIParseError},
    readers::{DiskReader, MIDIReader, RAMReader, TrackReader},
    track_parser::TrackParser,
};

pub enum RAMCache {
    Cache,
    NoCache,
    CacheIfPossible,
}

pub trait ReadSeek: Debug + Read + Seek {}
impl ReadSeek for File {}

#[derive(Debug)]
struct TrackPos {
    pos: u64,
    len: u32,
}

#[derive(Debug)]
pub struct MIDIFile {
    reader: Box<dyn MIDIReader>,
    track_positions: Vec<TrackPos>,

    format: u16,
    ppq: u16,
}

macro_rules! midi_error {
    ($val:expr) => {
        match $val {
            Ok(e) => Ok(e),
            Err(e) => Err(MIDILoadError::FilesystemError(e)),
        }
    };
}

impl MIDIFile {
    pub fn new_from_stream(
        reader: Box<dyn ReadSeek>,
        load_to_ram: bool,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<Self, MIDILoadError> {
        let reader = match load_to_ram {
            true => Box::new(RAMReader::new(reader)?) as Box<dyn MIDIReader>,
            false => Box::new(DiskReader::new(reader)?) as Box<dyn MIDIReader>,
        };

        MIDIFile::new_from_disk_reader(reader, read_progress)
    }

    pub fn new(
        filename: &str,
        load_to_ram: RAMCache,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<Self, MIDILoadError> {
        let reader = midi_error!(File::open(filename))?;
        let reader = match load_to_ram {
            RAMCache::Cache | RAMCache::CacheIfPossible => Box::new(RAMReader::new(Box::new(reader))?) as Box<dyn MIDIReader>,
            RAMCache::NoCache => Box::new(DiskReader::new(Box::new(reader))?) as Box<dyn MIDIReader>,
        };

        MIDIFile::new_from_disk_reader(reader, read_progress)
    }

    fn new_from_disk_reader(
        mut reader: Box<dyn MIDIReader>,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<Self, MIDILoadError> {
        reader.assert_header("MThd")?;

        let header_len = reader.read_value(4)?;

        if header_len != 6 {
            return Err(MIDILoadError::CorruptChunks);
        }

        let format = reader.read_value(2)? as u16;
        let _track_count_bad = reader.read_value(2)?;
        let ppq = reader.read_value(2)? as u16;

        let mut track_count = 0 as u32;
        let mut track_positions = Vec::<TrackPos>::new();
        while !reader.is_end()? {
            reader.assert_header("MTrk")?;
            track_count += 1;
            let len = reader.read_value(4)?;
            let pos = reader.get_position()?;
            track_positions.push(TrackPos { len, pos });
            reader.skip(len as u64)?;

            match read_progress {
                Some(progress) => progress(track_count),
                _ => {}
            };
        }

        track_positions.shrink_to_fit();
        Ok(MIDIFile {
            reader,
            ppq,
            format,
            track_positions,
        })
    }

    pub fn open_track_reader(&self, track: usize, ram_cache: bool) -> Box<dyn TrackReader> {
        let pos = &self.track_positions[track];
        self.reader.open_reader(pos.pos, pos.len as u64, ram_cache)
    }

    pub fn iter_all_tracks(
        &self,
        ram_cache: bool,
    ) -> impl Iterator<Item = impl Iterator<Item = Result<Event<u32>, MIDIParseError>>> {
        let mut tracks = Vec::new();
        for i in 0..self.track_count() {
            tracks.push(self.iter_track(i, ram_cache));
        }
        tracks.into_iter()
    }

    pub fn iter_track(
        &self,
        track: usize,
        ram_cache: bool,
    ) -> impl Iterator<Item = Result<Event<u32>, MIDIParseError>> {
        let reader = self.open_track_reader(track, ram_cache);
        let parser = TrackParser::new(reader);
        parser
    }

    pub fn ppq(&self) -> u16 {
        self.ppq
    }

    pub fn format(&self) -> u16 {
        self.format
    }

    pub fn track_count(&self) -> usize {
        self.track_positions.len()
    }
}
