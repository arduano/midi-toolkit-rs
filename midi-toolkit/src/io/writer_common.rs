use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    io::{self, Write},
};

use crate::events::SerializeEventWithDelta;

use super::MIDIWriteError;

pub fn encode_u16(val: u16) -> [u8; 2] {
    let mut bytes = [0; 2];
    bytes[0] = ((val >> 8) & 0xff) as u8;
    bytes[1] = (val & 0xff) as u8;
    bytes
}

pub fn encode_u32(val: u32) -> [u8; 4] {
    let mut bytes = [0; 4];
    bytes[0] = ((val >> 24) & 0xff) as u8;
    bytes[1] = ((val >> 16) & 0xff) as u8;
    bytes[2] = ((val >> 8) & 0xff) as u8;
    bytes[3] = (val & 0xff) as u8;
    bytes
}

pub fn write_midi_header(
    output: &mut dyn Write,
    format: u16,
    track_count: u16,
    ppq: u16,
) -> io::Result<()> {
    output.write_all(b"MThd")?;
    output.write_all(&encode_u32(6))?;
    output.write_all(&encode_u16(format))?;
    output.write_all(&encode_u16(track_count))?;
    output.write_all(&encode_u16(ppq))?;
    Ok(())
}

pub fn write_track_header(output: &mut dyn Write, length: u32) -> io::Result<()> {
    output.write_all(b"MTrk")?;
    output.write_all(&encode_u32(length))?;
    Ok(())
}

fn sorted_track_ids(tracks: &HashSet<i32>) -> Vec<i32> {
    let mut ids = tracks.iter().copied().collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

#[derive(Debug)]
pub struct OrderedTrackRegistry<T> {
    opened_tracks: HashSet<i32>,
    completed_tracks: BTreeSet<i32>,
    pending_tracks: BTreeMap<i32, T>,
    next_init_track: i32,
    next_flush_track: i32,
}

impl<T> OrderedTrackRegistry<T> {
    pub fn new() -> Self {
        Self {
            opened_tracks: HashSet::new(),
            completed_tracks: BTreeSet::new(),
            pending_tracks: BTreeMap::new(),
            next_init_track: 0,
            next_flush_track: 0,
        }
    }

    pub fn open_next_track(&mut self) -> Result<i32, MIDIWriteError> {
        let track_id = self.next_init_track;
        self.open_track(track_id)?;
        self.next_init_track += 1;
        Ok(track_id)
    }

    pub fn open_track(&mut self, track_id: i32) -> Result<(), MIDIWriteError> {
        if track_id < 0 {
            return Err(MIDIWriteError::InvalidTrackId { track_id });
        }

        if self.completed_tracks.contains(&track_id) || self.opened_tracks.contains(&track_id) {
            return Err(MIDIWriteError::TrackAlreadyOpened { track_id });
        }

        self.opened_tracks.insert(track_id);
        Ok(())
    }

    pub fn finish_track(&mut self, track_id: i32, payload: T) -> Result<(), MIDIWriteError> {
        if !self.opened_tracks.remove(&track_id) {
            return Err(MIDIWriteError::TrackAlreadyEnded { track_id });
        }

        self.completed_tracks.insert(track_id);
        self.pending_tracks.insert(track_id, payload);
        Ok(())
    }

    pub fn drain_ready_tracks(&mut self) -> Vec<(i32, T)> {
        let mut ready = Vec::new();
        while let Some(track) = self.pending_tracks.remove(&self.next_flush_track) {
            ready.push((self.next_flush_track, track));
            self.next_flush_track += 1;
        }
        ready
    }

    pub fn drain_all_tracks(&mut self) -> Vec<(i32, T)> {
        std::mem::take(&mut self.pending_tracks)
            .into_iter()
            .collect()
    }

    pub fn finalize_track_count(&self) -> Result<u16, MIDIWriteError> {
        if !self.opened_tracks.is_empty() {
            return Err(MIDIWriteError::OpenTracksRemaining {
                track_ids: sorted_track_ids(&self.opened_tracks),
            });
        }

        if let Some(&max_track) = self.completed_tracks.iter().next_back() {
            let mut missing = (0..max_track)
                .filter(|track_id| !self.completed_tracks.contains(track_id))
                .collect::<Vec<_>>();
            missing.sort_unstable();
            if !missing.is_empty() {
                return Err(MIDIWriteError::TrackGapsRemaining { track_ids: missing });
            }
        }

        let track_count = self.completed_tracks.len();
        if track_count > u16::MAX as usize {
            return Err(MIDIWriteError::TrackCountOverflow { track_count });
        }

        Ok(track_count as u16)
    }
}

impl<T> Default for OrderedTrackRegistry<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct LengthTrackingWriter<'a, W> {
    inner: &'a mut W,
    written: usize,
}

impl<'a, W> LengthTrackingWriter<'a, W> {
    fn new(inner: &'a mut W) -> Self {
        Self { inner, written: 0 }
    }
}

impl<W: Write> Write for LengthTrackingWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.inner.write(buf)?;
        self.written += written;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug)]
pub struct TrackByteSink<W> {
    track_id: i32,
    writer: Option<W>,
    length: u64,
}

impl<W> TrackByteSink<W> {
    pub fn new(track_id: i32, writer: W) -> Self {
        Self {
            track_id,
            writer: Some(writer),
            length: 0,
        }
    }

    pub fn track_id(&self) -> i32 {
        self.track_id
    }

    pub fn is_ended(&self) -> bool {
        self.writer.is_none()
    }

    pub fn writer_mut(&mut self) -> Result<&mut W, MIDIWriteError> {
        self.writer
            .as_mut()
            .ok_or(MIDIWriteError::TrackAlreadyEnded {
                track_id: self.track_id,
            })
    }

    fn note_write(&mut self, written: usize) -> Result<(), MIDIWriteError> {
        let new_length =
            self.length
                .checked_add(written as u64)
                .ok_or(MIDIWriteError::TrackLengthOverflow {
                    track_id: self.track_id,
                    length: u64::MAX,
                })?;

        if new_length > u32::MAX as u64 {
            return Err(MIDIWriteError::TrackLengthOverflow {
                track_id: self.track_id,
                length: new_length,
            });
        }

        self.length = new_length;
        Ok(())
    }
}

impl<W: Write> TrackByteSink<W> {
    pub fn write_event<T: SerializeEventWithDelta>(
        &mut self,
        event: T,
    ) -> Result<usize, MIDIWriteError> {
        let written = {
            let writer = self.writer_mut()?;
            let mut writer = LengthTrackingWriter::new(writer);
            event.serialize_event_with_delta(&mut writer)?;
            writer.written
        };
        self.note_write(written)?;
        Ok(written)
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
        self.note_write(bytes.len())?;
        self.writer_mut()?.write_all(bytes)?;
        Ok(bytes.len())
    }

    pub fn finish(&mut self) -> Result<(W, u32), MIDIWriteError> {
        if self.is_ended() {
            return Err(MIDIWriteError::TrackAlreadyEnded {
                track_id: self.track_id,
            });
        }

        self.write_bytes(&[0x00, 0xFF, 0x2F, 0x00])?;
        let length = self.length as u32;
        let writer = self
            .writer
            .take()
            .ok_or(MIDIWriteError::TrackAlreadyEnded {
                track_id: self.track_id,
            })?;
        Ok((writer, length))
    }
}

impl<W: Write> Write for TrackByteSink<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "track writer ended"))?;
        let written = writer.write(buf)?;
        self.note_write(written)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "track writer ended"))?
            .flush()
    }
}
