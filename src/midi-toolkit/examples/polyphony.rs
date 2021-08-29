use std::time::Instant;

use midi_toolkit::{
    events::{Event, MIDIEvent},
    io::MIDIFile,
    num::MIDINum,
    pipe,
    sequence::{event::merge_events_array, to_vec, unwrap_items},
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
    let file = MIDIFile::open_in_ram("D:/Midis/tau2.5.9.mid", None).unwrap();
    println!("Parsing midi...");
    let now = Instant::now();
    let mut poly: u64 = 0;
    let merged = pipe!(file.iter_all_tracks()|>to_vec()|>merge_events_array()|>unwrap_items());

    let mut max_poly: u64 = 0;

    for e in merged {
        match e {
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

    println!("Finished parsing midi, found {} polyphony", max_poly);
    println!("Elapsed {:?}", now.elapsed());
}
