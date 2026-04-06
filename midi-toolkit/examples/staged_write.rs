use std::{io::Write, thread};

use midi_toolkit::io::StagedMIDIWriter;

fn main() {
    let writer = StagedMIDIWriter::new("./staged-output.mid", 480).unwrap();
    println!(
        "Temporary parts file: {}",
        writer.temporary_parts_path().display()
    );

    let mut handles = Vec::new();
    for track_id in 0..3 {
        let mut track = writer.try_open_track(track_id).unwrap();
        handles.push(thread::spawn(move || {
            for step in 0..8 {
                let pitch = 60 + step as u8;
                track
                    .write_all(&[0x00, 0x90 | track_id as u8, pitch, 0x40])
                    .unwrap();
                track
                    .write_all(&[0x10, 0x80 | track_id as u8, pitch, 0x00])
                    .unwrap();
            }
            track.end().unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    writer.end().unwrap();
    println!(
        "Final MIDI written to {}",
        writer.destination_path().display()
    );
}
