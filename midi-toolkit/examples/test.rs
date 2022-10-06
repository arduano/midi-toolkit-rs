use std::time::Instant;

use midi_toolkit::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    io::MIDIFile,
    num::MIDINum,
    pipe,
    sequence::unwrap_items,
};

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open("D:/Midis/tau2.5.9.mid", None).unwrap();
    let now = Instant::now();
    let mut nc: u64 = 0;

    println!("Creating parsers...");
    let merged = pipe!(file.iter_all_track_events_merged()|>unwrap_items());
    println!("Parsing midi...");

    for e in merged {
        match e.as_event() {
            Event::NoteOn(_) => {
                dbg!(e);
                nc += 1;
                if nc > 10 {
                    break;
                }
            }
            _ => {}
        }
    }

    println!("Finished parsing midi, found {} notes", nc);
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
