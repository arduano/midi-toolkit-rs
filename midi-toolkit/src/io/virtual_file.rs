use std::{
    collections::BTreeMap,
    fs::{remove_file, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use super::VirtualFileError;

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug)]
struct Extent {
    offset: u64,
    len: u64,
}

#[derive(Debug)]
struct State {
    writer: Option<File>,
    next_offset: u64,
    next_stream_id: u64,
    streams: BTreeMap<u64, Vec<Extent>>,
}

#[derive(Debug)]
struct Inner {
    path: PathBuf,
    delete_on_drop: bool,
    state: Mutex<State>,
}

impl Inner {
    fn append(&self, stream_id: u64, bytes: &[u8]) -> Result<usize, VirtualFileError> {
        if bytes.is_empty() {
            return Ok(0);
        }

        let mut state = self.state.lock().unwrap();
        if !state.streams.contains_key(&stream_id) {
            return Err(VirtualFileError::UnknownStream { stream_id });
        }

        let offset = state.next_offset;
        {
            let writer = state.writer.as_mut().ok_or_else(|| {
                io::Error::new(io::ErrorKind::BrokenPipe, "backing file already closed")
            })?;
            writer.seek(SeekFrom::Start(offset))?;
            writer.write_all(bytes)?;
        }

        state.next_offset += bytes.len() as u64;
        state
            .streams
            .get_mut(&stream_id)
            .expect("stream presence was checked above")
            .push(Extent {
                offset,
                len: bytes.len() as u64,
            });

        Ok(bytes.len())
    }

    fn flush(&self) -> io::Result<()> {
        let mut state = self.state.lock().unwrap();
        match state.writer.as_mut() {
            Some(writer) => writer.flush(),
            None => Ok(()),
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if !self.delete_on_drop {
            return;
        }

        let writer = self.state.lock().unwrap().writer.take();
        drop(writer);
        let _ = remove_file(&self.path);
    }
}

fn create_unique_temp_file() -> io::Result<(File, PathBuf)> {
    let base = std::env::temp_dir();
    let pid = process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    for _ in 0..32 {
        let counter = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = base.join(format!(
            "midi-toolkit-interleaved-{pid}-{nanos}-{counter}.bin"
        ));

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path);

        match file {
            Ok(file) => return Ok((file, path)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "failed to allocate a unique temporary file",
    ))
}

#[derive(Clone, Debug)]
pub struct InterleavedTempFile {
    inner: Arc<Inner>,
}

impl InterleavedTempFile {
    pub fn new_temp() -> Result<Self, VirtualFileError> {
        let (file, path) = create_unique_temp_file()?;
        Ok(Self::new_inner(file, path, true))
    }

    pub fn new_at_path(path: impl AsRef<Path>) -> Result<Self, VirtualFileError> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        Ok(Self::new_inner(file, path, false))
    }

    pub fn new_temp_at_path(path: impl AsRef<Path>) -> Result<Self, VirtualFileError> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self::new_inner(file, path, true))
    }

    fn new_inner(file: File, path: PathBuf, delete_on_drop: bool) -> Self {
        Self {
            inner: Arc::new(Inner {
                path,
                delete_on_drop,
                state: Mutex::new(State {
                    writer: Some(file),
                    next_offset: 0,
                    next_stream_id: 0,
                    streams: BTreeMap::new(),
                }),
            }),
        }
    }

    pub fn path(&self) -> &Path {
        &self.inner.path
    }

    pub fn spawn_stream(&self) -> VirtualStreamWriter {
        let mut state = self.inner.state.lock().unwrap();
        let stream_id = state.next_stream_id;
        state.next_stream_id += 1;
        state.streams.insert(stream_id, Vec::new());

        VirtualStreamWriter {
            inner: Arc::clone(&self.inner),
            stream_id,
        }
    }

    pub fn stream_ids(&self) -> Vec<u64> {
        self.inner
            .state
            .lock()
            .unwrap()
            .streams
            .keys()
            .copied()
            .collect()
    }

    pub fn open_reader(&self, stream_id: u64) -> Result<VirtualStreamReader, VirtualFileError> {
        let extents = self
            .inner
            .state
            .lock()
            .unwrap()
            .streams
            .get(&stream_id)
            .cloned()
            .ok_or(VirtualFileError::UnknownStream { stream_id })?;

        let file = File::open(&self.inner.path)?;
        Ok(VirtualStreamReader {
            _inner: Arc::clone(&self.inner),
            file,
            extents,
            extent_index: 0,
            extent_offset: 0,
        })
    }

    pub fn flush(&self) -> Result<(), VirtualFileError> {
        Ok(self.inner.flush()?)
    }

    pub fn remove_backing_file(&self) -> Result<(), VirtualFileError> {
        let writer = self.inner.state.lock().unwrap().writer.take();
        drop(writer);

        match remove_file(&self.inner.path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug)]
pub struct VirtualStreamWriter {
    inner: Arc<Inner>,
    stream_id: u64,
}

impl VirtualStreamWriter {
    pub fn stream_id(&self) -> u64 {
        self.stream_id
    }

    pub fn flush_to_disk(&self) -> Result<(), VirtualFileError> {
        Ok(self.inner.flush()?)
    }
}

impl Write for VirtualStreamWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner
            .append(self.stream_id, buf)
            .map_err(|err| match err {
                VirtualFileError::FilesystemError(err) => err,
                other => io::Error::new(io::ErrorKind::NotFound, other),
            })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(Debug)]
pub struct VirtualStreamReader {
    _inner: Arc<Inner>,
    file: File,
    extents: Vec<Extent>,
    extent_index: usize,
    extent_offset: u64,
}

impl Read for VirtualStreamReader {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;

        while !buf.is_empty() && self.extent_index < self.extents.len() {
            let extent = self.extents[self.extent_index];
            if self.extent_offset >= extent.len {
                self.extent_index += 1;
                self.extent_offset = 0;
                continue;
            }

            let remaining = (extent.len - self.extent_offset) as usize;
            let to_read = remaining.min(buf.len());
            self.file
                .seek(SeekFrom::Start(extent.offset + self.extent_offset))?;

            let read = self.file.read(&mut buf[..to_read])?;
            if read == 0 {
                break;
            }

            total += read;
            self.extent_offset += read as u64;

            let (_, rest) = buf.split_at_mut(read);
            buf = rest;
        }

        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::{InterleavedTempFile, VirtualFileError};
    use std::{
        io::{Read, Write},
        thread,
        time::Instant,
    };

    fn assert_send<T: Send>() {}
    fn assert_send_sync<T: Send + Sync>() {}

    fn make_chunk(stream_index: usize, chunk_index: usize, chunk_len: usize) -> Vec<u8> {
        (0..chunk_len)
            .map(|byte_index| ((stream_index * 31 + chunk_index * 17 + byte_index) & 0xFF) as u8)
            .collect()
    }

    fn run_stress_case(stream_count: usize, chunks_per_stream: usize, chunk_len: usize) {
        let storage = InterleavedTempFile::new_temp().unwrap();
        let total_bytes = stream_count * chunks_per_stream * chunk_len;

        let write_started = Instant::now();
        let mut handles = Vec::new();
        for stream_index in 0..stream_count {
            let mut writer = storage.spawn_stream();
            handles.push(thread::spawn(move || {
                let stream_id = writer.stream_id();
                for chunk_index in 0..chunks_per_stream {
                    let chunk = make_chunk(stream_index, chunk_index, chunk_len);
                    writer.write_all(&chunk).unwrap();
                }
                writer.flush().unwrap();
                stream_id
            }));
        }

        let mut stream_ids = Vec::new();
        for handle in handles {
            stream_ids.push(handle.join().unwrap());
        }
        let write_elapsed = write_started.elapsed();

        let read_started = Instant::now();
        let mut read_handles = Vec::new();
        for (stream_index, stream_id) in stream_ids.into_iter().enumerate() {
            let storage = storage.clone();
            read_handles.push(thread::spawn(move || {
                let mut reader = storage.open_reader(stream_id).unwrap();
                for chunk_index in 0..chunks_per_stream {
                    let expected = make_chunk(stream_index, chunk_index, chunk_len);
                    let mut actual = vec![0; chunk_len];
                    reader.read_exact(&mut actual).unwrap();
                    assert_eq!(actual, expected);
                }

                let mut tail = [0_u8; 1];
                assert_eq!(reader.read(&mut tail).unwrap(), 0);
            }));
        }

        for handle in read_handles {
            handle.join().unwrap();
        }
        let read_elapsed = read_started.elapsed();

        let total_mib = total_bytes as f64 / (1024.0 * 1024.0);
        println!(
            "stress_case streams={stream_count} chunks_per_stream={chunks_per_stream} chunk_len={chunk_len} total_bytes={total_bytes} write={write_elapsed:?} ({:.2} MiB/s) read={read_elapsed:?} ({:.2} MiB/s)",
            total_mib / write_elapsed.as_secs_f64(),
            total_mib / read_elapsed.as_secs_f64(),
        );
    }

    #[test]
    fn manager_is_send_and_sync() {
        assert_send_sync::<InterleavedTempFile>();
    }

    #[test]
    fn stream_writer_is_send() {
        assert_send::<super::VirtualStreamWriter>();
    }

    #[test]
    fn writes_can_be_interleaved_and_read_back_independently() {
        let storage = InterleavedTempFile::new_temp().unwrap();

        let mut handles = Vec::new();
        for stream_index in 0..3 {
            let mut writer = storage.spawn_stream();
            handles.push(thread::spawn(move || {
                let stream_id = writer.stream_id();
                let parts = [
                    format!("stream-{stream_index}:alpha|"),
                    format!("stream-{stream_index}:beta|"),
                    format!("stream-{stream_index}:gamma"),
                ];

                let mut expected = String::new();
                for part in parts {
                    writer.write_all(part.as_bytes()).unwrap();
                    expected.push_str(&part);
                }

                writer.flush().unwrap();
                (stream_id, expected)
            }));
        }

        let mut expected_streams = Vec::new();
        for handle in handles {
            expected_streams.push(handle.join().unwrap());
        }
        expected_streams.sort_unstable_by_key(|(stream_id, _)| *stream_id);

        storage.flush().unwrap();
        assert_eq!(storage.stream_ids(), vec![0, 1, 2]);

        for (stream_id, expected) in expected_streams {
            let mut reader = storage.open_reader(stream_id).unwrap();
            let mut actual = String::new();
            reader.read_to_string(&mut actual).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn fragmented_stream_reads_preserve_exact_write_order() {
        let storage = InterleavedTempFile::new_temp().unwrap();
        let mut writer = storage.spawn_stream();
        let stream_id = writer.stream_id();

        let chunks = [
            b"header".as_slice(),
            b"|event-1|".as_slice(),
            b"event-2|".as_slice(),
            b"tail".as_slice(),
        ];
        let mut expected = Vec::new();
        for chunk in chunks {
            writer.write_all(chunk).unwrap();
            expected.extend_from_slice(chunk);
        }
        writer.flush().unwrap();

        let mut reader = storage.open_reader(stream_id).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn unknown_stream_returns_error() {
        let storage = InterleavedTempFile::new_temp().unwrap();
        let err = storage.open_reader(99).unwrap_err();
        assert!(matches!(
            err,
            VirtualFileError::UnknownStream { stream_id: 99 }
        ));
    }

    #[test]
    fn streams_can_be_read_in_parallel_after_writes_finish() {
        let storage = InterleavedTempFile::new_temp().unwrap();
        let mut streams = Vec::new();

        for stream_index in 0..6 {
            let mut writer = storage.spawn_stream();
            let stream_id = writer.stream_id();
            let mut expected = Vec::new();

            for chunk_index in 0..32 {
                let chunk = make_chunk(stream_index, chunk_index, 257);
                writer.write_all(&chunk).unwrap();
                expected.extend_from_slice(&chunk);
            }
            writer.flush().unwrap();
            streams.push((stream_id, expected));
        }

        let mut handles = Vec::new();
        for (stream_id, expected) in streams {
            let storage = storage.clone();
            handles.push(thread::spawn(move || {
                let mut reader = storage.open_reader(stream_id).unwrap();
                let mut actual = Vec::new();
                reader.read_to_end(&mut actual).unwrap();
                assert_eq!(actual, expected);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    #[ignore = "stress test: many tiny appends maximize lock and extent pressure"]
    fn stress_many_small_chunks() {
        run_stress_case(16, 20_000, 64);
    }

    #[test]
    #[ignore = "stress test: larger chunks emphasize raw IO throughput"]
    fn stress_large_chunks() {
        run_stress_case(8, 512, 16 * 1024);
    }
}
