use std::{
    io::{Read, Write},
    thread,
};

use midi_toolkit::io::InterleavedTempFile;

fn main() {
    let storage = InterleavedTempFile::new_temp().unwrap();
    let mut handles = Vec::new();

    for stream_index in 0..4 {
        let mut writer = storage.spawn_stream();
        handles.push(thread::spawn(move || {
            let stream_id = writer.stream_id();
            let chunks = [
                format!("track-{stream_index}:header|"),
                format!("track-{stream_index}:events|"),
                format!("track-{stream_index}:tail"),
            ];

            let mut expected = String::new();
            for chunk in chunks {
                writer.write_all(chunk.as_bytes()).unwrap();
                expected.push_str(&chunk);
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

    println!("Backing file: {}", storage.path().display());
    for (stream_id, expected) in expected_streams {
        let mut reader = storage.open_reader(stream_id).unwrap();
        let mut actual = String::new();
        reader.read_to_string(&mut actual).unwrap();

        assert_eq!(actual, expected);
        println!("stream {stream_id}: {actual}");
    }
}
