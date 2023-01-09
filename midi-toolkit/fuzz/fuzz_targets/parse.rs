#![no_main]

use libfuzzer_sys::fuzz_target;
//use cargo_libafl_helper::fuzz_target;

use midi_toolkit::{io::MIDIFile, pipe};
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data);
    if let Ok(file) = MIDIFile::open_from_stream_in_ram(cursor, None) {
        for track in file.iter_all_tracks() {
            for x in pipe!(track) {
                if x.is_err() {
                    break;
                }
            }
        }
    }
});
