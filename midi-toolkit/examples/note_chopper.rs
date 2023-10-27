#![feature(coroutines)]

use gen_iter::GenIter;
use midi_toolkit::{
    io::{MIDIFile, MIDIWriter},
    notes::{MIDINote, Note},
    pipe,
    sequence::{
        event::{filter_non_note_events, merge_events},
        events_to_notes,
        note::merge_notes_iterator,
        notes_to_events, to_vec_result, unwrap_items, wrap_ok,
    },
};

fn chop_note(note: Note<u64>, chop_size: u64) -> impl Iterator<Item = Note<u64>> {
    GenIter(move || {
        let mut pos = note.start;
        while pos < note.end() {
            let start = pos;
            let end = (pos + chop_size).min(note.end());
            yield Note {
                channel: note.channel,
                key: note.key,
                velocity: note.velocity,
                len: end - start,
                start,
            };
            pos = end;
        }
    })
}

fn main() {
    let file = MIDIFile::open("D:/Midis/tau2.5.9.mid", None).unwrap();

    let chop_size = file.ppq() as u64 / 16;

    let writer = MIDIWriter::new("./out.mid", file.ppq()).unwrap();

    for (i, track) in file.iter_all_tracks().enumerate() {
        println!("Chopping track {} of {}", i, file.track_count());

        let cached = pipe!(track|>to_vec_result()).unwrap();

        let non_note_events = pipe!(cached.iter().cloned()|>wrap_ok()|>filter_non_note_events());

        let notes = pipe!(cached.iter().cloned()|>wrap_ok()|>events_to_notes()|>unwrap_items());
        let chopped = notes.map(|n| wrap_ok(chop_note(n, chop_size)));
        let flattened = pipe!(chopped|>merge_notes_iterator()|>notes_to_events());

        // let notes = pipe!(cached.iter().cloned()|>wrap_ok()|>events_to_notes()|>unwrap_items());
        // let flattened = pipe!(notes|>wrap_ok()|>notes_to_events());

        let merged = merge_events(flattened, non_note_events);

        writer
            .open_next_track()
            .write_events_iter(pipe!(merged|>unwrap_items()))
            .unwrap();
    }
}
