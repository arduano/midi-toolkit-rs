use std::{
    ffi::OsString,
    fs::File,
    io::{self, copy, Write},
    path::{Path, PathBuf},
    process,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::events::SerializeEventWithDelta;

use super::{
    writer_common::{write_midi_header, write_track_header, OrderedTrackRegistry, TrackByteSink},
    InterleavedTempFile, MIDIWriteError, VirtualStreamWriter,
};

static STAGED_PARTS_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WriterPhase {
    Active,
    Finalizing,
    Ended,
}

#[derive(Debug)]
struct WrittenTrack {
    stream_id: u64,
    length: u32,
}

#[derive(Debug)]
struct WriterState {
    tracks: OrderedTrackRegistry<WrittenTrack>,
    format: u16,
    ppq: u16,
    phase: WriterPhase,
}

#[derive(Debug)]
struct Inner {
    destination: PathBuf,
    temp_storage: InterleavedTempFile,
    state: Mutex<WriterState>,
}

pub struct StagedMIDIWriter {
    inner: Arc<Inner>,
}

pub struct StagedTrackWriter {
    inner: Arc<Inner>,
    stream_id: u64,
    track: TrackByteSink<VirtualStreamWriter>,
}

fn create_sidecar_storage(destination: &Path) -> Result<InterleavedTempFile, MIDIWriteError> {
    let pid = process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    for _ in 0..32 {
        let counter = STAGED_PARTS_COUNTER.fetch_add(1, Ordering::Relaxed);
        let suffix = format!("{pid:x}{nanos:x}{counter:x}");
        let mut file_name = destination
            .file_name()
            .map(|name| name.to_os_string())
            .unwrap_or_else(|| OsString::from("output.mid"));
        file_name.push(format!(".parts.{suffix}"));
        let temp_path = destination.with_file_name(file_name);

        match InterleavedTempFile::new_temp_at_path(&temp_path) {
            Ok(storage) => return Ok(storage),
            Err(super::VirtualFileError::FilesystemError(err))
                if err.kind() == io::ErrorKind::AlreadyExists =>
            {
                continue;
            }
            Err(err) => return Err(err.into()),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "failed to allocate a unique staged MIDI sidecar",
    )
    .into())
}

impl StagedMIDIWriter {
    pub fn new(destination: impl AsRef<Path>, ppq: u16) -> Result<Self, MIDIWriteError> {
        let destination = destination.as_ref().to_path_buf();
        let temp_storage = create_sidecar_storage(&destination)?;

        Ok(Self {
            inner: Arc::new(Inner {
                destination,
                temp_storage,
                state: Mutex::new(WriterState {
                    tracks: OrderedTrackRegistry::new(),
                    format: 1,
                    ppq,
                    phase: WriterPhase::Active,
                }),
            }),
        })
    }

    pub fn destination_path(&self) -> &Path {
        &self.inner.destination
    }

    pub fn temporary_parts_path(&self) -> &Path {
        self.inner.temp_storage.path()
    }

    pub fn write_ppq(&self, ppq: u16) -> Result<(), MIDIWriteError> {
        let mut state = self.inner.state.lock().unwrap();
        if state.phase != WriterPhase::Active {
            return Err(MIDIWriteError::WriterEnded);
        }

        state.ppq = ppq;
        Ok(())
    }

    pub fn write_format(&self, format: u16) -> Result<(), MIDIWriteError> {
        let mut state = self.inner.state.lock().unwrap();
        if state.phase != WriterPhase::Active {
            return Err(MIDIWriteError::WriterEnded);
        }

        state.format = format;
        Ok(())
    }

    pub fn try_open_next_track(&self) -> Result<StagedTrackWriter, MIDIWriteError> {
        let mut state = self.inner.state.lock().unwrap();
        if state.phase != WriterPhase::Active {
            return Err(MIDIWriteError::WriterEnded);
        }

        let track_id = state.tracks.open_next_track()?;

        let writer = self.inner.temp_storage.spawn_stream();
        let stream_id = writer.stream_id();

        Ok(StagedTrackWriter {
            inner: Arc::clone(&self.inner),
            stream_id,
            track: TrackByteSink::new(track_id, writer),
        })
    }

    pub fn try_open_track(&self, track_id: i32) -> Result<StagedTrackWriter, MIDIWriteError> {
        let mut state = self.inner.state.lock().unwrap();
        if state.phase != WriterPhase::Active {
            return Err(MIDIWriteError::WriterEnded);
        }
        state.tracks.open_track(track_id)?;

        let writer = self.inner.temp_storage.spawn_stream();
        let stream_id = writer.stream_id();

        Ok(StagedTrackWriter {
            inner: Arc::clone(&self.inner),
            stream_id,
            track: TrackByteSink::new(track_id, writer),
        })
    }

    pub fn is_ended(&self) -> bool {
        self.inner.state.lock().unwrap().phase == WriterPhase::Ended
    }

    pub fn try_end(&self) -> Result<(), MIDIWriteError> {
        let (tracks, track_count, format, ppq) = {
            let mut state = self.inner.state.lock().unwrap();
            match state.phase {
                WriterPhase::Ended | WriterPhase::Finalizing => {
                    return Err(MIDIWriteError::WriterEnded);
                }
                WriterPhase::Active => {}
            }

            let track_count = state.tracks.finalize_track_count()?;

            state.phase = WriterPhase::Finalizing;
            (
                state.tracks.drain_all_tracks(),
                track_count,
                state.format,
                state.ppq,
            )
        };

        let result = (|| -> Result<(), MIDIWriteError> {
            self.inner.temp_storage.flush()?;

            let mut output = File::create(&self.inner.destination)?;
            write_midi_header(&mut output, format, track_count, ppq)?;

            for (_track_id, track) in tracks {
                write_track_header(&mut output, track.length)?;

                let mut reader = self.inner.temp_storage.open_reader(track.stream_id)?;
                copy(&mut reader, &mut output)?;
            }

            output.flush()?;
            self.inner.temp_storage.remove_backing_file()?;
            Ok(())
        })();

        let mut state = self.inner.state.lock().unwrap();
        state.phase = if result.is_ok() {
            WriterPhase::Ended
        } else {
            WriterPhase::Active
        };

        result
    }

    pub fn end(&self) -> Result<(), MIDIWriteError> {
        self.try_end()
    }
}

impl StagedTrackWriter {
    pub fn is_ended(&self) -> bool {
        self.track.is_ended()
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

    pub fn end(&mut self) -> Result<(), MIDIWriteError> {
        let track_id = self.track.track_id();
        let (_writer, length) = self.track.finish()?;

        let mut state = self.inner.state.lock().unwrap();
        state.tracks.finish_track(
            track_id,
            WrittenTrack {
                stream_id: self.stream_id,
                length,
            },
        )?;
        Ok(())
    }
}

impl Write for StagedTrackWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.track.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.track.flush()
    }
}

impl std::fmt::Debug for StagedTrackWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StagedTrackWriter")
            .field("track_id", &self.track.track_id())
            .field("is_ended", &self.is_ended())
            .finish()
    }
}

impl Drop for StagedTrackWriter {
    fn drop(&mut self) {
        if !self.is_ended() {
            let _ = self.end();
        }
    }
}

impl Drop for StagedMIDIWriter {
    fn drop(&mut self) {
        if !self.is_ended() {
            let _ = self.end();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StagedMIDIWriter;
    use crate::io::MIDIWriteError;
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn assert_send<T: Send>() {}
    fn assert_send_sync<T: Send + Sync>() {}

    fn unique_output_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("midi-toolkit-{test_name}-{nanos}.mid"))
    }

    fn read_file(path: &Path) -> Vec<u8> {
        fs::read(path).unwrap()
    }

    #[test]
    fn writer_is_send_sync_and_track_writer_is_send() {
        assert_send_sync::<StagedMIDIWriter>();
        assert_send::<super::StagedTrackWriter>();
    }

    #[test]
    fn finalizes_tracks_in_track_id_order_and_cleans_up_sidecar() {
        let output = unique_output_path("staged-order");
        let writer = StagedMIDIWriter::new(&output, 480).unwrap();
        let temp_path = writer.temporary_parts_path().to_path_buf();

        let mut track_two = writer.try_open_track(2).unwrap();
        let mut track_zero = writer.try_open_track(0).unwrap();
        let mut track_one = writer.try_open_track(1).unwrap();

        track_two.write_all(&[0x00, 0xC0, 0x02]).unwrap();
        track_zero.write_all(&[0x00, 0x90, 0x3C, 0x64]).unwrap();
        track_one.write_all(&[0x00, 0x80, 0x3C, 0x00]).unwrap();

        track_two.end().unwrap();
        track_zero.end().unwrap();
        track_one.end().unwrap();

        assert!(temp_path.exists());
        writer.end().unwrap();
        assert!(!temp_path.exists());

        let bytes = read_file(&output);
        assert_eq!(&bytes[0..4], b"MThd");
        assert_eq!(&bytes[8..10], &[0x00, 0x01]);
        assert_eq!(&bytes[10..12], &[0x00, 0x03]);
        assert_eq!(&bytes[12..14], &[0x01, 0xE0]);

        assert_eq!(&bytes[14..18], b"MTrk");
        assert_eq!(&bytes[18..22], &[0x00, 0x00, 0x00, 0x08]);
        assert_eq!(
            &bytes[22..30],
            &[0x00, 0x90, 0x3C, 0x64, 0x00, 0xFF, 0x2F, 0x00]
        );

        assert_eq!(&bytes[30..34], b"MTrk");
        assert_eq!(&bytes[34..38], &[0x00, 0x00, 0x00, 0x08]);
        assert_eq!(
            &bytes[38..46],
            &[0x00, 0x80, 0x3C, 0x00, 0x00, 0xFF, 0x2F, 0x00]
        );

        assert_eq!(&bytes[46..50], b"MTrk");
        assert_eq!(&bytes[50..54], &[0x00, 0x00, 0x00, 0x07]);
        assert_eq!(&bytes[54..61], &[0x00, 0xC0, 0x02, 0x00, 0xFF, 0x2F, 0x00]);

        let _ = fs::remove_file(output);
    }

    #[test]
    fn tracks_can_be_written_from_threads_and_finalized_afterwards() {
        let output = unique_output_path("staged-threaded");
        let writer = StagedMIDIWriter::new(&output, 960).unwrap();

        let mut handles = Vec::new();
        for track_id in 0..4 {
            let mut track = writer.try_open_track(track_id).unwrap();
            handles.push(thread::spawn(move || {
                for note in 0..64 {
                    let velocity = 32 + (note % 64) as u8;
                    track
                        .write_all(&[0x00, 0x90, 36 + track_id as u8, velocity])
                        .unwrap();
                }
                track.end().unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        writer.end().unwrap();
        let bytes = read_file(&output);
        assert_eq!(&bytes[10..12], &[0x00, 0x04]);
        assert_eq!(&bytes[12..14], &[0x03, 0xC0]);

        let track_headers = bytes.windows(4).filter(|window| *window == b"MTrk").count();
        assert_eq!(track_headers, 4);

        let _ = fs::remove_file(output);
    }

    #[test]
    fn end_rejects_track_gaps() {
        let output = unique_output_path("staged-gap");
        let writer = StagedMIDIWriter::new(&output, 480).unwrap();
        let mut track = writer.try_open_track(1).unwrap();
        track.end().unwrap();

        let err = writer.end().unwrap_err();
        assert!(matches!(
            err,
            MIDIWriteError::TrackGapsRemaining { ref track_ids } if track_ids == &[0]
        ));

        let _ = fs::remove_file(output);
    }
}
