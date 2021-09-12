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

#[derive(Debug)]
pub enum MIDIParseError {
    CorruptEvent,
    UnexpectedTrackEnd,
    FilesystemError(std::io::Error),
}

impl From<std::io::Error> for MIDIParseError {
    fn from(e: std::io::Error) -> Self {
        MIDIParseError::FilesystemError(e)
    }
}

#[derive(Debug)]
pub enum MIDIWriteError {
    FilesystemError(std::io::Error),
}

impl From<std::io::Error> for MIDIWriteError {
    fn from(e: std::io::Error) -> Self {
        MIDIWriteError::FilesystemError(e)
    }
}
