use std::time::Instant;

use midi_toolkit::{
    events::{Event, MIDIEventEnum},
    io::MIDIFile,
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
        if let Event::NoteOn(_) = e.as_event() {
            dbg!(e);
            nc += 1;
            if nc > 10 {
                break;
            }
        }
    }

    println!("Finished parsing midi, found {nc} notes");
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
