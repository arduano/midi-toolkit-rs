use crossbeam_channel::{bounded, unbounded, Sender};
use std::{
    io::{self, SeekFrom},
    sync::Arc,
    thread::{self, JoinHandle},
};

use super::{
    errors::{MIDILoadError, MIDIParseError},
    midi_file::ReadSeek,
};

use std::fmt::Debug;
#[derive(Debug)]
pub struct DiskReader {
    reader: BufferReadProvider,
    length: u64,
}

#[derive(Debug)]
pub struct RAMReader {
    bytes: Arc<Vec<u8>>,
    pos: usize,
}

pub struct ReadCommand {
    destination: Arc<Sender<Result<Vec<u8>, io::Error>>>,
    buffer: Vec<u8>,
    start: u64,
    length: usize,
}

#[derive(Debug)]
pub struct BufferReadProvider {
    thread: JoinHandle<()>,
    send: Sender<ReadCommand>,
}

impl BufferReadProvider {
    pub fn new<T: 'static + ReadSeek>(mut reader: T) -> BufferReadProvider {
        let (snd, rcv) = unbounded::<ReadCommand>();

        let handle = thread::spawn(move || {
            let mut read =
                move |mut buffer: Vec<u8>, start: u64, length: usize| -> Result<Vec<u8>, io::Error> {
                    reader.seek(SeekFrom::Start(start))?;
                    let (sub, _) = buffer.split_at_mut(length);
                    reader.read_exact(sub)?;
                    Ok(buffer)
                };

            loop {
                match rcv.recv() {
                    Err(_) => return,
                    Ok(cmd) => match read(cmd.buffer, cmd.start, cmd.length) {
                        Ok(buf) => {
                            cmd.destination.send(Ok(buf)).ok();
                        }
                        Err(e) => {
                            cmd.destination.send(Err(e)).ok();
                        }
                    },
                }
            }
        });

        BufferReadProvider {
            send: snd,
            thread: handle,
        }
    }

    pub fn send_read_command(
        &self,
        destination: Arc<Sender<Result<Vec<u8>, io::Error>>>,
        buffer: Vec<u8>,
        start: u64,
        length: usize,
    ) {
        let cmd = ReadCommand {
            destination,
            buffer,
            start,
            length,
        };

        self.send.send(cmd).unwrap();
    }

    pub fn read_sync(&self, buf: Vec<u8>, start: u64) -> Result<Vec<u8>, io::Error> {
        let (send, receive) = bounded::<Result<Vec<u8>, io::Error>>(1);

        let len = buf.len();
        self.send_read_command(Arc::new(send), buf, start, len);

        receive.recv().unwrap()
    }
}

macro_rules! midi_error {
    ($val:expr) => {
        match $val {
            Ok(e) => Ok(e),
            Err(e) => Err(MIDILoadError::FilesystemError(e)),
        }
    };
}

fn get_reader_len<T: ReadSeek>(reader: &mut T) -> Result<u64, MIDILoadError> {
    let mut get = || {
        let pos = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;
        return Ok(pos);
    };

    midi_error!(get())
}

impl DiskReader {
    pub fn new<T: 'static + ReadSeek>(mut reader: T) -> Result<DiskReader, MIDILoadError> {
        let len = get_reader_len(&mut reader);
        let reader = BufferReadProvider::new(reader);

        match len {
            Err(e) => Err(e),
            Ok(length) => Ok(DiskReader { reader, length }),
        }
    }
}

impl RAMReader {
    pub fn new<T: ReadSeek>(mut reader: T) -> Result<RAMReader, MIDILoadError> {
        let len = get_reader_len(&mut reader);

        match len {
            Err(e) => Err(e),
            Ok(length) => {
                let max_supported: u64 = 2147483648;
                if length > max_supported {
                    panic!(
                        "The maximum length allowed for a memory loaded MIDI file is {}",
                        max_supported
                    );
                }

                let mut bytes = vec![0; length as usize];
                midi_error!(reader.read(&mut bytes))?;
                Ok(RAMReader {
                    bytes: Arc::new(bytes),
                    pos: 0,
                })
            }
        }
    }

    pub fn read_byte(&mut self) -> Result<u8, MIDILoadError> {
        let b = self.bytes.get(self.pos);
        self.pos += 1;
        match b {
            Some(v) => Ok(*v),
            None => Err(MIDILoadError::CorruptChunks),
        }
    }
}

pub trait MIDIReader: Debug {
    fn read_bytes_to(
        &self,
        pos: u64,
        bytes: Vec<u8>,
    ) -> Result<Vec<u8>, MIDILoadError>;
    fn read_bytes(&self, pos: u64, count: usize) -> Result<Vec<u8>, MIDILoadError> {
        let bytes = vec![0u8; count];

        self.read_bytes_to(pos, bytes)
    }

    fn len(&self) -> u64;

    fn open_reader(&self, start: u64, len: u64, ram_cache: bool) -> FullRamTrackReader;
}

impl MIDIReader for DiskReader {
    fn open_reader(&self, _start: u64, _len: u64, _ram_cache: bool) -> FullRamTrackReader {
        todo!()
    }

    fn read_bytes_to(
        &self,
        pos: u64,
        bytes: Vec<u8>,
    ) -> Result<Vec<u8>, MIDILoadError> {
        midi_error!(self.reader.read_sync(bytes, pos))
    }

    fn len(&self) -> u64 {
        self.length
    }
}

impl MIDIReader for RAMReader {
    fn open_reader<'a>(&self, start: u64, len: u64, _ram_cache: bool) -> FullRamTrackReader {
        FullRamTrackReader {
            pos: start as usize,
            end: (start + len) as usize,
            bytes: self.bytes.clone(),
        }
    }

    fn read_bytes_to(
        &self,
        pos: u64,
        mut bytes: Vec<u8>,
    ) -> Result<Vec<u8>, MIDILoadError> {
        let count = bytes.len();
        if pos + count as u64 > self.len() {
            return Err(MIDILoadError::CorruptChunks);
        }

        for i in 0..count {
            bytes[i] = self.bytes[pos as usize + i];
        }

        Ok(bytes)
    }

    fn len(&self) -> u64 {
        self.bytes.len() as u64
    }
}

pub trait TrackReader {
    fn read(&mut self) -> Result<u8, MIDIParseError>;
}

pub struct FullRamTrackReader {
    bytes: Arc<Vec<u8>>,
    pos: usize,
    end: usize,
}

impl TrackReader for FullRamTrackReader {
    fn read(&mut self) -> Result<u8, MIDIParseError> {
        if self.pos == self.end {
            return Err(MIDIParseError::UnexpectedTrackEnd);
        }
        let b = self.bytes[self.pos];
        self.pos += 1;
        Ok(b)
    }
}
