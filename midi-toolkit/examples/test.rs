use std::time::Instant;

use midi_toolkit::{
    events::{Event, MIDIEvent},
    io::MIDIFile,
    num::MIDINum,
    pipe,
    sequence::unwrap_items,
};

pub fn boxed<
    T: MIDINum,
    E: MIDIEvent<T>,
    Err,
    I: 'static + Iterator<Item = Result<E, Err>> + Sized,
>(
    iter: I,
) -> Box<impl Iterator<Item = Result<E, Err>>> {
    Box::new(iter)
}

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open("F:/Fast MIDIs/The Nuker 4 F2.mid", None).unwrap();
    let now = Instant::now();
    let mut nc: u64 = 0;

    println!("Creating parsers...");
    let merged = pipe!(file.iter_all_events_merged()|>unwrap_items());
    println!("Parsing midi...");

    for e in merged {
        match e {
            Event::NoteOn(_) => nc += 1,
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
