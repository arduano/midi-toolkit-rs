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

fn sorted_track_ids(tracks: &HashSet<i32>) -> Vec<i32> {
    let mut ids = tracks.iter().copied().collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn encode_u16(val: u16) -> [u8; 2] {
    let mut bytes = [0; 2];
    bytes[0] = ((val >> 8) & 0xff) as u8;
    bytes[1] = (val & 0xff) as u8;
    bytes
}

fn encode_u32(val: u32) -> [u8; 4] {
    let mut bytes = [0; 4];
    bytes[0] = ((val >> 24) & 0xff) as u8;
    bytes[1] = ((val >> 16) & 0xff) as u8;
    bytes[2] = ((val >> 8) & 0xff) as u8;
    bytes[3] = (val & 0xff) as u8;
    bytes
}

fn flush_track(writer: &mut dyn WriteSeek, mut output: QueuedOutput) -> Result<(), io::Error> {
    writer.write_all("MTrk".as_bytes())?;
    writer.write_all(&encode_u32(output.length))?;
    copy(&mut output.write, writer)?;
    Ok(())
}

impl MIDIWriter {
    pub fn new(filename: &str, ppq: u16) -> Result<MIDIWriter, MIDIWriteError> {
        let reader = File::create(filename)?;
        MIDIWriter::new_from_stream(Box::new(reader), ppq)
    }

    pub fn new_from_stream(
        mut output: Box<dyn WriteSeek>,
        ppq: u16,
    ) -> Result<MIDIWriter, MIDIWriteError> {
        output.seek(SeekFrom::Start(0))?;
        output.write_all("MThd".as_bytes())?;
        output.write_all(&encode_u32(6))?;
        output.write_all(&encode_u16(1))?;
        output.write_all(&encode_u16(0))?;
        output.write_all(&encode_u16(ppq))?;

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

    #[deprecated(note = "use new_from_stream")]
    pub fn new_from_stram(
        output: Box<dyn WriteSeek>,
        ppq: u16,
    ) -> Result<MIDIWriter, MIDIWriteError> {
        Self::new_from_stream(output, ppq)
    }

    fn with_writer<R>(
        &self,
        f: impl FnOnce(&mut dyn WriteSeek) -> Result<R, io::Error>,
    ) -> Result<R, MIDIWriteError> {
        let output = self.output.as_ref().ok_or(MIDIWriteError::WriterEnded)?;
        let mut output = output.lock().unwrap();
        Ok(f(output.as_mut())?)
    }

    fn write_u16_at(&self, at: u64, val: u16) -> Result<(), MIDIWriteError> {
        self.with_writer(|output| {
            let pos = output.stream_position()?;
            output.seek(SeekFrom::Start(at))?;
            output.write_all(&encode_u16(val))?;
            output.seek(SeekFrom::Start(pos))?;
            Ok(())
        })
    }

    pub fn write_ppq(&self, ppq: u16) -> Result<(), MIDIWriteError> {
        self.write_u16_at(12, ppq)
    }

    pub fn write_format(&self, format: u16) -> Result<(), MIDIWriteError> {
        self.write_u16_at(8, format)
    }

    fn write_ntrks(&self, track_count: u16) -> Result<(), MIDIWriteError> {
        self.write_u16_at(10, track_count)
    }

    pub fn try_open_next_track(&self) -> Result<TrackWriter<'_>, MIDIWriteError> {
        let mut tracks = self.tracks.lock().unwrap();
        if self.output.is_none() {
            return Err(MIDIWriteError::WriterEnded);
        }

        let track_id = tracks.next_init_track;
        if tracks.written_tracks.contains(&track_id) || tracks.opened_tracks.contains(&track_id) {
            return Err(MIDIWriteError::TrackAlreadyOpened { track_id });
        }

        tracks.next_init_track += 1;
        tracks.opened_tracks.insert(track_id);

        Ok(TrackWriter {
            midi_writer: self,
            track_id,
            writer: Some(Cursor::new(Vec::new())),
        })
    }

    #[deprecated(note = "use try_open_next_track")]
    pub fn open_next_track(&self) -> TrackWriter<'_> {
        self.try_open_next_track()
            .expect("failed to open next track")
    }

    pub fn try_open_track(&self, track_id: i32) -> Result<TrackWriter<'_>, MIDIWriteError> {
        if self.output.is_none() {
            return Err(MIDIWriteError::WriterEnded);
        }

        let mut tracks = self.tracks.lock().unwrap();
        if tracks.written_tracks.contains(&track_id) || tracks.opened_tracks.contains(&track_id) {
            return Err(MIDIWriteError::TrackAlreadyOpened { track_id });
        }

        tracks.opened_tracks.insert(track_id);

        Ok(TrackWriter {
            midi_writer: self,
            track_id,
            writer: Some(Cursor::new(Vec::new())),
        })
    }

    #[deprecated(note = "use try_open_track")]
    pub fn open_track(&self, track_id: i32) -> TrackWriter<'_> {
        self.try_open_track(track_id).expect("failed to open track")
    }

    pub fn try_end(&mut self) -> Result<(), MIDIWriteError> {
        if self.is_ended() {
            return Err(MIDIWriteError::WriterEnded);
        }

        let (open_tracks, missing_tracks, track_count) = {
            let tracks = self.tracks.lock().unwrap();
            if !tracks.opened_tracks.is_empty() {
                (
                    Some(sorted_track_ids(&tracks.opened_tracks)),
                    None,
                    tracks.written_tracks.len(),
                )
            } else if !tracks.queued_writes.is_empty() {
                let max_track = *tracks
                    .queued_writes
                    .keys()
                    .max()
                    .expect("queued_writes checked to be non-empty");
                let mut missing = (0..max_track)
                    .filter(|track_id| !tracks.written_tracks.contains(track_id))
                    .collect::<Vec<_>>();
                missing.sort_unstable();
                (None, Some(missing), tracks.written_tracks.len())
            } else {
                (None, None, tracks.written_tracks.len())
            }
        };

        if let Some(track_ids) = open_tracks {
            return Err(MIDIWriteError::OpenTracksRemaining { track_ids });
        }

        if let Some(track_ids) = missing_tracks {
            return Err(MIDIWriteError::TrackGapsRemaining { track_ids });
        }

        if track_count > u16::MAX as usize {
            return Err(MIDIWriteError::TrackCountOverflow { track_count });
        }

        self.write_ntrks(track_count as u16)?;
        self.output.take();

        Ok(())
    }

    pub fn end(&mut self) -> Result<(), MIDIWriteError> {
        self.try_end()
    }

    pub fn is_ended(&self) -> bool {
        self.output.is_none()
    }
}

impl<'a> TrackWriter<'a> {
    fn writer_mut(&mut self) -> Result<&mut Cursor<Vec<u8>>, MIDIWriteError> {
        let track_id = self.track_id;
        self.writer
            .as_mut()
            .ok_or(MIDIWriteError::TrackAlreadyEnded { track_id })
    }

    pub fn end(&mut self) -> Result<(), MIDIWriteError> {
        if self.is_ended() {
            return Err(MIDIWriteError::TrackAlreadyEnded {
                track_id: self.track_id,
            });
        }

        self.write_bytes(&[0x00, 0xFF, 0x2F, 0x00])?;

        let length = self
            .writer
            .as_ref()
            .expect("writer presence was checked above")
            .position() as u32;

        let mut status = self.midi_writer.tracks.lock().unwrap();
        if !status.opened_tracks.remove(&self.track_id) {
            return Err(MIDIWriteError::TrackAlreadyEnded {
                track_id: self.track_id,
            });
        }
        status.written_tracks.insert(self.track_id);

        let mut writer = self
            .writer
            .take()
            .ok_or(MIDIWriteError::TrackAlreadyEnded {
                track_id: self.track_id,
            })?;
        writer.seek(SeekFrom::Start(0))?;

        status.queued_writes.insert(
            self.track_id,
            QueuedOutput {
                write: Box::new(writer),
                length,
            },
        );

        if self.track_id == status.next_write_track {
            let output = self
                .midi_writer
                .output
                .as_ref()
                .ok_or(MIDIWriteError::WriterEnded)?;
            let mut writer = output.lock().unwrap();
            loop {
                let next_write_track = status.next_write_track;
                match status.queued_writes.remove_entry(&next_write_track) {
                    None => break,
                    Some(output) => {
                        flush_track(writer.as_mut(), output.1)?;
                        status.next_write_track += 1;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn is_ended(&self) -> bool {
        self.writer.is_none()
    }

    pub fn get_writer_mut(&mut self) -> &mut impl Write {
        self.writer_mut()
            .expect("Tried to write to TrackWriter after .end() was called")
    }

    pub fn write_event<T: SerializeEventWithDelta>(
        &mut self,
        event: T,
    ) -> Result<usize, MIDIWriteError> {
        let writer = self.writer_mut()?;
        event.serialize_event_with_delta(writer)
    }

    pub fn write_events_iter<T: SerializeEventWithDelta>(
        &mut self,
        events: impl Iterator<Item = T>,
    ) -> Result<usize, MIDIWriteError> {
        let mut count = 0;
        for event in events {
            count += self.write_event(event)?;
        }
        Ok(count)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, MIDIWriteError> {
        let writer = self.writer_mut()?;
        writer.write_all(bytes)?;
        Ok(bytes.len())
    }
}

impl<'a> std::fmt::Debug for TrackWriter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackWriter")
            .field("track_id", &self.track_id)
            .field("is_ended", &self.is_ended())
            .finish()
    }
}

impl<'a> Drop for TrackWriter<'a> {
    fn drop(&mut self) {
        if !self.is_ended() {
            let _ = self.end();
        }
    }
}

impl Drop for MIDIWriter {
    fn drop(&mut self) {
        if !self.is_ended() {
            let _ = self.end();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MIDIWriter, WriteSeek};
    use crate::io::MIDIWriteError;
    use std::{
        io::{Cursor, Seek, SeekFrom, Write},
        sync::{Arc, Mutex},
    };

    #[derive(Clone, Default)]
    struct SharedCursor {
        inner: Arc<Mutex<Cursor<Vec<u8>>>>,
    }

    impl SharedCursor {
        fn bytes(&self) -> Vec<u8> {
            self.inner.lock().unwrap().get_ref().clone()
        }
    }

    impl Write for SharedCursor {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.inner.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.inner.lock().unwrap().flush()
        }
    }

    impl Seek for SharedCursor {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            self.inner.lock().unwrap().seek(pos)
        }
    }

    impl WriteSeek for SharedCursor {}

    #[test]
    fn is_ended_matches_writer_lifecycle() {
        let shared = SharedCursor::default();
        let mut writer = MIDIWriter::new_from_stream(Box::new(shared), 480).unwrap();
        assert!(!writer.is_ended());

        let mut track = writer.try_open_next_track().unwrap();
        assert!(!track.is_ended());

        track.end().unwrap();
        assert!(track.is_ended());
        drop(track);
        assert!(!writer.is_ended());

        writer.end().unwrap();
        assert!(writer.is_ended());
    }

    #[test]
    fn drop_still_finalizes_open_writers() {
        let shared = SharedCursor::default();
        {
            let writer = MIDIWriter::new_from_stream(Box::new(shared.clone()), 480).unwrap();
            let mut track = writer.try_open_next_track().unwrap();
            track.write_bytes(&[0x00, 0x90, 0x3C, 0x40]).unwrap();
        }

        let bytes = shared.bytes();
        assert_eq!(&bytes[0..4], b"MThd");
        assert_eq!(&bytes[10..12], &[0x00, 0x01]);
        assert_eq!(&bytes[14..18], b"MTrk");
        assert_eq!(&bytes[18..22], &[0x00, 0x00, 0x00, 0x08]);
        assert_eq!(
            &bytes[22..30],
            &[0x00, 0x90, 0x3C, 0x40, 0x00, 0xFF, 0x2F, 0x00]
        );
    }

    #[test]
    fn try_open_track_reports_duplicates_and_writer_end() {
        let shared = SharedCursor::default();
        let mut writer = MIDIWriter::new_from_stream(Box::new(shared), 480).unwrap();

        let mut track = writer.try_open_next_track().unwrap();
        let err = writer
            .try_open_track(0)
            .expect_err("duplicate track should fail");
        assert!(matches!(
            err,
            MIDIWriteError::TrackAlreadyOpened { track_id: 0 }
        ));

        track.end().unwrap();
        drop(track);
        writer.end().unwrap();
        let err = writer
            .try_open_track(3)
            .expect_err("ended writer should fail");
        assert!(matches!(err, MIDIWriteError::WriterEnded));
    }

    #[test]
    fn try_end_reports_missing_track_gaps() {
        let shared = SharedCursor::default();
        let mut writer = MIDIWriter::new_from_stream(Box::new(shared), 480).unwrap();
        let mut track = writer.try_open_track(1).unwrap();
        track.end().unwrap();
        drop(track);
        let gap_err = writer
            .try_end()
            .expect_err("missing track 0 should prevent end");
        assert!(matches!(
            gap_err,
            MIDIWriteError::TrackGapsRemaining { ref track_ids } if track_ids == &[0]
        ));
    }

    #[test]
    fn write_bytes_writes_all_bytes() {
        let shared = SharedCursor::default();
        {
            let writer = MIDIWriter::new_from_stream(Box::new(shared.clone()), 480).unwrap();
            let mut track = writer.try_open_next_track().unwrap();
            track.write_bytes(&[1, 2, 3, 4]).unwrap();
            track.end().unwrap();
        }

        let bytes = shared.bytes();
        assert!(bytes.windows(4).any(|window| window == [1, 2, 3, 4]));
    }
}
