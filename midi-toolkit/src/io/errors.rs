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
    CorruptEvent,
    UnexpectedTrackEnd,
    FilesystemError(#[from] std::io::Error),
}

impl std::fmt::Display for MIDIParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MIDIParseError::CorruptEvent => write!(f, "Corrupt event"),
            MIDIParseError::UnexpectedTrackEnd => write!(f, "Unexpected track end"),
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
