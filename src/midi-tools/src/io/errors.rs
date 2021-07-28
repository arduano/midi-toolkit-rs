#[derive(Debug)]
pub enum MIDILoadError {
    CorruptChunks,
    FilesystemError(std::io::Error),
    FileTooBig,
}

#[derive(Debug)]
pub enum MIDIParseError {
    CorruptEvent,
    UnexpectedTrackEnd,
}
