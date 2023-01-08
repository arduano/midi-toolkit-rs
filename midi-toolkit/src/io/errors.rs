use thiserror::Error;

#[derive(Debug)]
pub enum MIDILoadError {
    CorruptChunks,
    FilesystemError(std::io::Error),
    FileTooBig,
}

impl From<std::io::Error> for MIDILoadError {
    fn from(e: std::io::Error) -> Self {
        MIDILoadError::FilesystemError(e)
    }
}

#[derive(Debug, Error)]
pub enum MIDIParseError {
    CorruptEvent {
        track_number: Option<u32>,
        position: u64,
    },
    UnexpectedTrackEnd {
        track_number: Option<u32>,
        track_start: u64,
        expected_track_end: u64,
        found_track_end: u64,
    },
    FilesystemError(#[from] std::io::Error),
}

impl std::fmt::Display for MIDIParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MIDIParseError::CorruptEvent {
                track_number,
                position,
            } => match track_number {
                Some(track_number) => write!(
                    f,
                    "Corrupt event (track {track_number}, position: {position:#06x})",
                ),
                None => write!(f, "Corrupt event (position: {position:#06x})", ),
            },
            MIDIParseError::UnexpectedTrackEnd {
                track_number,
                track_start,
                expected_track_end,
                found_track_end,
            } => match track_number {
                Some(track_number) => write!(f, "Unexpected track end (track {track_number}, track start: {track_start:#06x}, expected end: {expected_track_end:#06x}, found end: {found_track_end:#06x})"),
                None => write!(f, "Unexpected track end (track start: {track_start:#06x}, expected end: {expected_track_end:#06x}, found end: {found_track_end:#06x})")
            },
            MIDIParseError::FilesystemError(e) => write!(f, "Filesystem error: {e}"),
        }
    }
}

#[derive(Debug, Error)]
pub enum MIDIWriteError {
    FilesystemError(#[from] std::io::Error),
}

impl std::fmt::Display for MIDIWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MIDIWriteError::FilesystemError(e) => write!(f, "Filesystem error: {e}"),
        }
    }
}
