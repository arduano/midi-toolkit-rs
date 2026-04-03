use std::time::Instant;

use midi_toolkit::{events::Event, io::MIDIFile, prelude::*};

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open_in_ram("D:/Midis/tau2.5.9.mid", None).unwrap();
    println!("Parsing midi...");
    let now = Instant::now();
    let mut poly: u64 = 0;
    let merged = file.iter_all_tracks().merge_all().unwrap_items();

    let mut max_poly: u64 = 0;

    for e in merged {
        match *e {
            Event::NoteOn(_) => {
                poly += 1;
                if poly > max_poly {
                    max_poly = poly;
                }
            }
            Event::NoteOff(_) => poly -= 1,
            _ => {}
        }
    }

    println!("Finished parsing midi, found {max_poly} polyphony");
    println!("Elapsed {:?}", now.elapsed());
}
