// use std::{
//     fs::File,
//     io::{self, Read, Seek, SeekFrom},
//     sync::Arc,
// };

// use crate::errors::MIDILoadError;

// pub struct DiskReader {
//     reader: File,
//     length: u64,
// }

// pub struct RAMReader {
//     bytes: Arc<Vec<u8>>,
//     pos: usize,
// }

// fn e<T>(val: Result<T, io::Error>) -> Result<T, MIDILoadError> {
//     match val {
//         Ok(e) => Ok(e),
//         Err(_) => Err(MIDILoadError::UnknownFilesystemError),
//     }
// }

// fn get_reader_len(reader: &mut File) -> Result<u64, MIDILoadError> {
//     let mut get = || {
//         let pos = reader.seek(SeekFrom::End(0))?;
//         reader.seek(SeekFrom::Start(0))?;
//         return Ok(pos);
//     };

//     e(get())
// }

// impl DiskReader {
//     pub fn new(filename: &str) -> Result<DiskReader, MIDILoadError> {
//         let reader_maybe = File::open(filename);
//         if reader_maybe.is_err() {
//             return Err(MIDILoadError::NotFound);
//         }

//         let mut reader = reader_maybe.unwrap();

//         let len = get_reader_len(&mut reader);

//         match len {
//             Err(_) => Err(MIDILoadError::UnknownFilesystemError),
//             Ok(length) => Ok(DiskReader { reader, length }),
//         }
//     }
// }

// impl RAMReader {
//     pub fn new(filename: &str) -> Result<RAMReader, MIDILoadError> {
//         let reader_maybe = File::open(filename);
//         if reader_maybe.is_err() {
//             return Err(MIDILoadError::NotFound);
//         }

//         let mut reader = reader_maybe.unwrap();

//         let len = get_reader_len(&mut reader);

//         match len {
//             Err(_) => Err(MIDILoadError::UnknownFilesystemError),
//             Ok(length) => {
//                 let mut bytes = vec![0; length as usize];
//                 e(reader.read(&mut bytes))?;
//                 Ok(RAMReader {
//                     bytes: Arc::new(bytes),
//                     pos: 0,
//                 })
//             }
//         }
//     }

//     pub fn read_byte(&mut self) -> Result<u8, MIDILoadError> {
//         let b = self.bytes.get(self.pos);
//         self.pos += 1;
//         match b {
//             Some(v) => Ok(*v),
//             None => Err(MIDILoadError::CorruptChunks),
//         }
//     }
// }

// pub trait MIDIReader {
//     fn assert_header(&mut self, text: &str) -> Result<(), MIDILoadError>;
//     fn read_value(&mut self, bytes: i32) -> Result<u32, MIDILoadError>;
//     fn get_position(&mut self) -> Result<u64, MIDILoadError>;
//     fn is_end(&mut self) -> Result<bool, MIDILoadError>;
//     fn skip(&mut self, bytes: u64) -> Result<u64, MIDILoadError>;

//     fn open_reader(&self, start: u64, len: u64, ram_cache: bool) -> Box<dyn TrackReader>;
// }

// impl MIDIReader for DiskReader {
//     fn assert_header(&mut self, text: &str) -> Result<(), MIDILoadError> {
//         let reader = &mut self.reader;
//         let chars = text.as_bytes();
//         let mut bytes = vec![0 as u8; chars.len()];
//         let read = reader.read_exact(&mut bytes);

//         if read.is_err() {
//             return Err(MIDILoadError::CorruptChunks);
//         }

//         for i in 0..chars.len() {
//             if chars[i] != bytes[i] {
//                 return Err(MIDILoadError::CorruptChunks);
//             }
//         }
//         return Ok(());
//     }

//     fn read_value(&mut self, bytes: i32) -> Result<u32, MIDILoadError> {
//         let reader = &mut self.reader;

//         let mut b = vec![0 as u8; bytes as usize];
//         let read = e(reader.read_exact(&mut b));

//         match read {
//             Err(e) => Err(e),
//             Ok(_) => {
//                 let mut num: u32 = 0;
//                 for v in b {
//                     num = (num << 8) + v as u32;
//                 }
//                 Ok(num)
//             }
//         }
//     }

//     fn get_position(&mut self) -> Result<u64, MIDILoadError> {
//         e(self.reader.stream_position())
//     }

//     fn is_end(&mut self) -> Result<bool, MIDILoadError> {
//         Ok(self.get_position()? == self.length)
//     }

//     fn skip(&mut self, bytes: u64) -> Result<u64, MIDILoadError> {
//         let pos = self.get_position()?;
//         let mut to = pos as u64 + bytes;
//         if to > self.length as u64 {
//             to = self.length as u64;
//         }
//         e(self.reader.seek(SeekFrom::Start(to)))
//     }

//     fn open_reader(&self, start: u64, len: u64, ram_cache: bool) -> Box<dyn TrackReader> {
//         todo!()
//     }
// }

// impl MIDIReader for RAMReader {
//     fn assert_header(&mut self, text: &str) -> Result<(), MIDILoadError> {
//         let chars = text.as_bytes();

//         for i in 0..chars.len() {
//             let read = self.read_byte()?;
//             if chars[i] != read {
//                 return Err(MIDILoadError::CorruptChunks);
//             }
//         }
//         return Ok(());
//     }

//     fn read_value(&mut self, bytes: i32) -> Result<u32, MIDILoadError> {
//         let mut num: u32 = 0;
//         for _ in 0..bytes {
//             num = (num << 8) + self.read_byte()? as u32;
//         }
//         Ok(num)
//     }

//     fn get_position(&mut self) -> Result<u64, MIDILoadError> {
//         Ok(self.pos as u64)
//     }

//     fn is_end(&mut self) -> Result<bool, MIDILoadError> {
//         Ok(self.pos == self.bytes.len())
//     }

//     fn skip(&mut self, bytes: u64) -> Result<u64, MIDILoadError> {
//         let pos = self.get_position()?;
//         let mut to = pos as u64 + bytes;
//         let len = self.bytes.len();
//         if to > len as u64 {
//             to = len as u64;
//         }
//         self.pos = to as usize;
//         Ok(to)
//     }

//     fn open_reader<'a>(&self, start: u64, len: u64, _ram_cache: bool) -> Box<dyn TrackReader> {
//         Box::new(FullRamTrackReader {
//             pos: start as usize,
//             end: (start + len) as usize,
//             bytes: self.bytes.clone(),
//         })
//     }
// }

// pub trait TrackReader {
//     fn read(&mut self) -> Result<u8, MIDILoadError>;
// }

// pub struct FullRamTrackReader {
//     bytes: Arc<Vec<u8>>,
//     pos: usize,
//     end: usize,
// }

// impl TrackReader for FullRamTrackReader {
//     fn read(&mut self) -> Result<u8, MIDILoadError> {
//         if self.pos == self.end {
//             return Err(MIDILoadError::OutOfBoundsError);
//         }
//         let b = self.bytes[self.pos];
//         self.pos += 1;
//         Ok(b)
//     }
// }
