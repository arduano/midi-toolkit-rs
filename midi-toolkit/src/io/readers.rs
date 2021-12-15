use crossbeam_channel::{bounded, unbounded, Sender};
use std::{
    io::{self, SeekFrom},
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::DelayedReceiver;

use super::{
    errors::{MIDILoadError, MIDIParseError},
    midi_file::ReadSeek,
};

use std::fmt::Debug;
#[derive(Debug)]
pub struct DiskReader {
    reader: Arc<BufferReadProvider>,
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
    _thread: JoinHandle<()>,
    send: Sender<ReadCommand>,
}

impl BufferReadProvider {
    pub fn new<T: 'static + ReadSeek>(mut reader: T) -> BufferReadProvider {
        let (snd, rcv) = unbounded::<ReadCommand>();

        let handle = thread::spawn(move || {
            let mut read = move |mut buffer: Vec<u8>,
                                 start: u64,
                                 length: usize|
                  -> Result<Vec<u8>, io::Error> {
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
            _thread: handle,
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

fn get_reader_len<T: ReadSeek>(reader: &mut T) -> Result<u64, MIDILoadError> {
    let pos = reader.seek(SeekFrom::End(0))?;
    reader.seek(SeekFrom::Start(0))?;
    Ok(pos)
}

impl DiskReader {
    pub fn new<T: 'static + ReadSeek>(mut reader: T) -> Result<DiskReader, MIDILoadError> {
        let len = get_reader_len(&mut reader);
        let reader = BufferReadProvider::new(reader);

        match len {
            Err(e) => Err(e),
            Ok(length) => Ok(DiskReader {
                reader: Arc::new(reader),
                length,
            }),
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
                reader.read(&mut bytes)?;
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

pub trait MIDIReader<T: TrackReader>: Debug {
    fn read_bytes_to(&self, pos: u64, bytes: Vec<u8>) -> Result<Vec<u8>, MIDILoadError>;
    fn read_bytes(&self, pos: u64, count: usize) -> Result<Vec<u8>, MIDILoadError> {
        let bytes = vec![0u8; count];

        self.read_bytes_to(pos, bytes)
    }

    fn len(&self) -> u64;

    fn open_reader(&self, start: u64, len: u64) -> T;
}

impl MIDIReader<DiskTrackReader> for DiskReader {
    fn open_reader(&self, start: u64, len: u64) -> DiskTrackReader {
        DiskTrackReader::new(self.reader.clone(), start, len)
    }

    fn read_bytes_to(&self, pos: u64, bytes: Vec<u8>) -> Result<Vec<u8>, MIDILoadError> {
        Ok(self.reader.read_sync(bytes, pos)?)
    }

    fn len(&self) -> u64 {
        self.length
    }
}

impl MIDIReader<FullRamTrackReader> for RAMReader {
    fn open_reader<'a>(&self, start: u64, len: u64) -> FullRamTrackReader {
        FullRamTrackReader {
            pos: start as usize,
            end: (start + len) as usize,
            bytes: self.bytes.clone(),
        }
    }

    fn read_bytes_to(&self, pos: u64, mut bytes: Vec<u8>) -> Result<Vec<u8>, MIDILoadError> {
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

pub trait TrackReader: Send + Sync {
    fn read(&mut self) -> Result<u8, MIDIParseError>;
}

pub struct DiskTrackReader {
    reader: Arc<BufferReadProvider>,
    pos: u64,                    // Relative to midi
    len: u64,                    //
    buffer: Option<Vec<u8>>,     //
    buffer_start: u64,           // Relative to pos
    buffer_pos: usize,           // Relative buffer start
    unrequested_data_start: u64, // Relative to pos

    receiver: DelayedReceiver<Result<Vec<u8>, io::Error>>,
    receiver_sender: Option<Arc<Sender<Result<Vec<u8>, io::Error>>>>, // Becomes None when there's nothing left to read
}

pub struct FullRamTrackReader {
    bytes: Arc<Vec<u8>>,
    pos: usize,
    end: usize,
}

impl FullRamTrackReader {
    pub fn new(bytes: Arc<Vec<u8>>, pos: usize, end: usize) -> FullRamTrackReader {
        FullRamTrackReader { bytes, pos, end }
    }

    pub fn new_from_vec(bytes: Vec<u8>) -> FullRamTrackReader {
        let len = bytes.len();
        FullRamTrackReader {
            bytes: Arc::new(bytes),
            pos: 0,
            end: len,
        }
    }
}

impl TrackReader for FullRamTrackReader {
    #[inline(always)]
    fn read(&mut self) -> Result<u8, MIDIParseError> {
        if self.pos == self.end {
            return Err(MIDIParseError::UnexpectedTrackEnd);
        }
        let b = self.bytes[self.pos];
        self.pos += 1;
        Ok(b)
    }
}

impl DiskTrackReader {
    fn finished_sending_reads(&self) -> bool {
        self.unrequested_data_start == self.len
    }

    fn next_buffer_req_length(&self) -> usize {
        (self.len - self.unrequested_data_start).min(1 << 19) as usize
    }

    fn send_next_read(&mut self, buffer: Option<Vec<u8>>) {
        if self.finished_sending_reads() {
            self.receiver_sender.take();
            return;
        }

        let mut next_len = self.next_buffer_req_length();

        let buffer = match buffer {
            None => vec![0u8; next_len],
            Some(b) => b,
        };

        next_len = next_len.min(buffer.len());

        self.reader.send_read_command(
            self.receiver_sender.clone().unwrap(),
            buffer,
            self.unrequested_data_start + self.pos,
            next_len,
        );

        self.unrequested_data_start += next_len as u64;
    }

    fn receive_next_buffer(&mut self) -> Option<Result<Vec<u8>, MIDIParseError>> {
        match self.receiver.recv() {
            Ok(v) => match v {
                Ok(v) => Some(Ok(v)),
                Err(e) => Some(Err(e.into())),
            },
            Err(_) => None,
        }
    }

    pub fn new(reader: Arc<BufferReadProvider>, start: u64, len: u64) -> DiskTrackReader {
        let buffer_count = 3;

        let (send, receive) = unbounded();
        let send = Arc::new(send);

        let mut reader = DiskTrackReader {
            reader: reader,
            pos: start,
            len: len as u64,
            buffer: None,
            buffer_start: 0,
            buffer_pos: 0,
            unrequested_data_start: 0,
            receiver: DelayedReceiver::new(receive),
            receiver_sender: Some(send),
        };

        for _ in 0..buffer_count {
            reader.send_next_read(None);
        }

        reader.receiver.wait_first();

        reader
    }
}

impl TrackReader for DiskTrackReader {
    #[inline(always)]
    fn read(&mut self) -> Result<u8, MIDIParseError> {
        match self.buffer {
            None => {
                if let Some(next) = self.receive_next_buffer() {
                    self.buffer = Some(next?);
                } else {
                    return Err(MIDIParseError::UnexpectedTrackEnd);
                }
            }
            Some(_) => {}
        }

        let buffer = self.buffer.as_ref().unwrap();
        let byte = buffer[self.buffer_pos];

        self.buffer_pos += 1;
        if self.buffer_pos == buffer.len() {
            let buffer = self.buffer.take().unwrap();
            self.buffer_start += buffer.len() as u64;
            self.buffer_pos = 0;
            self.send_next_read(Some(buffer));
        }

        Ok(byte)
    }
}
