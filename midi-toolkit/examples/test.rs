use std::time::Instant;

use midi_toolkit::{
    events::{Event, MIDIEventEnum},
    io::MIDIFile,
    pipe,
    sequence::unwrap_items,
};

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open("/run/media/d/Midis/The Quarantine Project.mid", None).unwrap();
    let now = Instant::now();
    let mut nc: u64 = 0;

    println!("Creating parsers...");
    let merged = pipe!(file.iter_all_track_events_merged()|>unwrap_items());
    println!("Parsing midi...");

    for e in merged {
        if let Event::NoteOn(_) = e.as_event() {
            nc += 1;
        }
    }

    println!("Finished parsing midi, found {nc} notes");
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
