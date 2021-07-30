use std::time::Instant;

use midi_tools::{events::{Event, MIDIEvent}, io::{midi_file::MIDIFile, midi_writer::MIDIWriter, readers::RAMReader}, num::MIDINum, pipe, sequence::{
        unwrap_items,
    }};

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
    let file = MIDIFile::<RAMReader>::new(
        "D:/Midis/tau2.5.9.mid",
        None,
    )
    .unwrap();
    let writer = MIDIWriter::new("./out.mid", file.ppq()).unwrap();
    println!("Parsing midi...");
    let now = Instant::now();
    let mut nc: u64 = 0;
    for track in file.iter_all_tracks(true) {
        let mut track_writer = writer.open_next_track();
        for e in pipe!(track|>unwrap_items())
        {
            match e {
                Event::NoteOn(_) => nc += 1,
                _ => {}
            }
            track_writer.write_event(e).unwrap();
        }
    }
    // let merged = pipe!(file.iter_all_tracks(true)|>to_vec()|>merge_events_array()|>unwrap_items());

    // for e in merged {
    //     match e {
    //         Event::NoteOn(_) => nc += 1,
    //         _ => {}
    //     }
    // }
    println!("Finished parsing midi, found {} notes", nc);
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
