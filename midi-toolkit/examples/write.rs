use std::time::Instant;

use midi_toolkit::{
    events::Event,
    io::{MIDIFile, MIDIWriter},
    pipe,
    sequence::{event::merge_events_array, to_vec, unwrap_items},
};

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open_in_ram("D:/Midis/tau2.5.9.mid", None).unwrap();
    let writer = MIDIWriter::new("./out.mid", file.ppq()).unwrap();
    println!("Parsing midi...");
    let now = Instant::now();

    let mut nc: u64 = 0;
    {
        let mut track_writer = writer.open_next_track();
        // let merged = pipe!(file.iter_all_tracks()|>to_vec()|>merge_events_array()|>unwrap_items());
        let converted = file.iter_all_tracks();
        // .map(|track| pipe!(track|>events_to_notes()|>notes_to_events()));
        let merged = pipe!(converted|>to_vec()|>merge_events_array()|>unwrap_items());
        for e in merged {
            if let Event::NoteOn(_) = *e {
                nc += 1
            }
            track_writer.write_event(e).unwrap();
        }
    }

    println!("Finished parsing midi, found {nc} notes");
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
