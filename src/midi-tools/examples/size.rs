use std::time::Instant;

use midi_tools::{
    events::Event,
    io::midi_file::{MIDIFile, RAMCache},
    pipe,
    sequence::unwrap_items,
};

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::new(
        "D:/Midis/Ra Ra Rasputin Ultimate Black MIDI Final.mid",
        RAMCache::CacheIfPossible,
        None,
    )
    .unwrap();
    println!("Parsing midi...");
    let now = Instant::now();
    let mut nc: u64 = 0;
    for track in file.iter_all_tracks(true) {
        for e in pipe!(track|>unwrap_items()) {
            match e {
                Event::NoteOn(e) => nc += 1,
                _ => {}
            }
        }
    }
    println!("Finished parsing midi, found {} notes", nc);
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
