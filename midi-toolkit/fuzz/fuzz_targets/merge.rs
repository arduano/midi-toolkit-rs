#![no_main]

use libfuzzer_sys::fuzz_target;

use midi_toolkit::{
    io::{MIDIFile, MIDIWriter},
    pipe,
    sequence::{event::merge_events_array, to_vec},
};
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data);
    if let Ok(file) = MIDIFile::open_from_stream_in_ram(cursor, None) {
        let merged = pipe!(file.iter_all_tracks()|>to_vec()|>merge_events_array());
        for x in merged {
            if x.is_err() {
                break;
            }
        }
    }
});
