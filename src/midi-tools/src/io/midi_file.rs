use std::{fs::File, io::{Read, Seek}, marker::PhantomData};

use crate::events::Event;
use std::fmt::Debug;

use super::{
    errors::{MIDILoadError, MIDIParseError},
    readers::{
        DiskReader, DiskTrackReader, FullRamTrackReader, MIDIReader, RAMReader, TrackReader,
    },
    track_parser::TrackParser,
};

pub trait ReadSeek: Debug + Read + Seek + Send {}
impl ReadSeek for File {}

#[derive(Debug)]
struct TrackPos {
    pos: u64,
    len: u32,
}

#[derive(Debug)]
pub struct MIDIFileParser<T: MIDIReader<R>, R: TrackReader> {
    reader: T,
    track_positions: Vec<TrackPos>,

    format: u16,
    ppq: u16,

    _t: PhantomData<R>
}

macro_rules! midi_error {
    ($val:expr) => {
        match $val {
            Ok(e) => Ok(e),
            Err(e) => Err(MIDILoadError::FilesystemError(e)),
        }
    };
}

impl<T: MIDIReader<R>, R: TrackReader> MIDIFileParser<T, R> {
    fn new_from_disk_reader(
        reader: T,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<Self, MIDILoadError> {
        fn bytes_to_val(bytes: &[u8]) -> u32 {
            assert!(bytes.len() <= 4);
            let mut num: u32 = 0;
            for b in bytes {
                num = (num << 8) + *b as u32;
            }

            num
        }

        fn read_header<T: MIDIReader<R>, R: TrackReader>(
            reader: &T,
            pos: u64,
            text: &str,
        ) -> Result<u32, MIDILoadError> {
            assert!(text.len() == 4);

            let bytes = reader.read_bytes(pos, 8)?;

            let (header, len) = bytes.split_at(4);

            let chars = text.as_bytes();

            for i in 0..chars.len() {
                if chars[i] != header[i] {
                    return Err(MIDILoadError::CorruptChunks);
                }
            }

            Ok(bytes_to_val(len))
        }

        let mut pos = 0u64;

        let header_len = read_header(&reader, pos, "MThd")?;
        pos += 8;
        if header_len != 6 {
            return Err(MIDILoadError::CorruptChunks);
        }

        let (format, ppq) = {
            let header_data = reader.read_bytes(pos, 6)?;
            pos += 6;
            let (format_bytes, rest) = header_data.split_at(2);
            let (_, ppq_bytes) = rest.split_at(2);
            (
                bytes_to_val(format_bytes) as u16,
                bytes_to_val(ppq_bytes) as u16,
            )
        };

        let mut track_count = 0 as u32;
        let mut track_positions = Vec::<TrackPos>::new();
        while pos != reader.len() {
            let len = read_header(&reader, pos, "MTrk")?;
            pos += 8;
            track_count += 1;
            track_positions.push(TrackPos { len, pos });
            pos += len as u64;

            match read_progress {
                Some(progress) => progress(track_count),
                _ => {}
            };
        }

        track_positions.shrink_to_fit();
        Ok(MIDIFileParser {
            reader,
            ppq,
            format,
            track_positions,
            _t: PhantomData,
        })
    }

    pub fn open_track_reader(&self, track: usize) -> R {
        let pos = &self.track_positions[track];
        self.reader.open_reader(pos.pos, pos.len as u64)
    }

    pub fn iter_all_tracks(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = Result<Event<u64>, MIDIParseError>>> {
        let mut tracks = Vec::new();
        for i in 0..self.track_count() {
            tracks.push(self.iter_track(i));
        }
        tracks.into_iter()
    }

    pub fn iter_track(
        &self,
        track: usize,
    ) -> impl Iterator<Item = Result<Event<u64>, MIDIParseError>> {
        let reader = self.open_track_reader(track);
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

pub struct MIDIFile;

impl MIDIFile {
    pub fn open(
        filename: &str,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<MIDIFileParser<DiskReader, DiskTrackReader>, MIDILoadError> {
        let reader = midi_error!(File::open(filename))?;
        let reader = DiskReader::new(reader)?;

        MIDIFileParser::new_from_disk_reader(reader, read_progress)
    }

    pub fn open_in_ram(
        filename: &str,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<MIDIFileParser<RAMReader, FullRamTrackReader>, MIDILoadError> {
        let reader = midi_error!(File::open(filename))?;
        let reader = RAMReader::new(reader)?;

        MIDIFileParser::new_from_disk_reader(reader, read_progress)
    }

    pub fn open_from_stream(
        filename: &str,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<MIDIFileParser<DiskReader, DiskTrackReader>, MIDILoadError> {
        let reader = midi_error!(File::open(filename))?;
        let reader = DiskReader::new(reader)?;

        MIDIFileParser::new_from_disk_reader(reader, read_progress)
    }

    pub fn open_from_stream_in_ram(
        filename: &str,
        read_progress: Option<&dyn Fn(u32)>,
    ) -> Result<MIDIFileParser<RAMReader, FullRamTrackReader>, MIDILoadError> {
        let reader = midi_error!(File::open(filename))?;
        let reader = RAMReader::new(reader)?;

        MIDIFileParser::new_from_disk_reader(reader, read_progress)
    }
}
