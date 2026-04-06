use std::{
    fs::File,
    io::{self, copy, Cursor, Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

use crate::events::SerializeEventWithDelta;

use super::{
    errors::MIDIWriteError,
    writer_common::{
        encode_u16, write_midi_header, write_track_header, OrderedTrackRegistry, TrackByteSink,
    },
};

pub trait WriteSeek: Write + Seek {}
impl WriteSeek for File {}
impl WriteSeek for Cursor<Vec<u8>> {}

pub struct QueuedOutput {
    write: Box<dyn Read>,
    length: u32,
}

pub struct MIDIWriter {
    output: Option<Mutex<Box<dyn WriteSeek>>>,
    tracks: Mutex<OrderedTrackRegistry<QueuedOutput>>,
}

pub struct TrackWriter<'a> {
    midi_writer: &'a MIDIWriter,
    track: TrackByteSink<Cursor<Vec<u8>>>,
}

fn flush_track(writer: &mut dyn WriteSeek, mut output: QueuedOutput) -> Result<(), io::Error> {
    write_track_header(writer, output.length)?;
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
        write_midi_header(output.as_mut(), 1, 0, ppq)?;

        Ok(MIDIWriter {
            output: Some(Mutex::new(output)),
            tracks: Mutex::new(OrderedTrackRegistry::new()),
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

        let track_id = tracks.open_next_track()?;

        Ok(TrackWriter {
            midi_writer: self,
            track: TrackByteSink::new(track_id, Cursor::new(Vec::new())),
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
        tracks.open_track(track_id)?;

        Ok(TrackWriter {
            midi_writer: self,
            track: TrackByteSink::new(track_id, Cursor::new(Vec::new())),
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

        let track_count = self.tracks.lock().unwrap().finalize_track_count()?;
        self.write_ntrks(track_count)?;
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
        self.track.writer_mut()
    }

    pub fn end(&mut self) -> Result<(), MIDIWriteError> {
        let track_id = self.track.track_id();
        let (mut writer, length) = self.track.finish()?;
        writer.seek(SeekFrom::Start(0))?;

        let queued_outputs = {
            let mut status = self.midi_writer.tracks.lock().unwrap();
            status.finish_track(
                track_id,
                QueuedOutput {
                    write: Box::new(writer),
                    length,
                },
            )?;
            status.drain_ready_tracks()
        };

        if !queued_outputs.is_empty() {
            let output = self
                .midi_writer
                .output
                .as_ref()
                .ok_or(MIDIWriteError::WriterEnded)?;
            let mut writer = output.lock().unwrap();
            for (_, queued_output) in queued_outputs {
                flush_track(writer.as_mut(), queued_output)?;
            }
        }

        Ok(())
    }

    pub fn is_ended(&self) -> bool {
        self.track.is_ended()
    }

    pub fn get_writer_mut(&mut self) -> &mut impl Write {
        self.writer_mut()
            .expect("Tried to write to TrackWriter after .end() was called")
    }

    pub fn write_event<T: SerializeEventWithDelta>(
        &mut self,
        event: T,
    ) -> Result<usize, MIDIWriteError> {
        self.track.write_event(event)
    }

    pub fn write_events_iter<T: SerializeEventWithDelta>(
        &mut self,
        events: impl Iterator<Item = T>,
    ) -> Result<usize, MIDIWriteError> {
        self.track.write_events_iter(events)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, MIDIWriteError> {
        self.track.write_bytes(bytes)
    }
}

impl<'a> std::fmt::Debug for TrackWriter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrackWriter")
            .field("track_id", &self.track.track_id())
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
