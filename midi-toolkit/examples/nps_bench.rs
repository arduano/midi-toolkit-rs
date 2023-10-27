use std::time::Instant;

use midi_toolkit::{
    events::{Event, MIDIEventEnum},
    io::MIDIFile,
    pipe,
    sequence::{
        event::{cancel_tempo_events, scale_event_time},
        unwrap_items, TimeCaster,
    },
};

pub fn main() {
    println!("Opening midi...");
    let midi = MIDIFile::open_in_ram(
        "/mnt/fat/Midis/Ra Ra Rasputin Ultimate Black MIDI Final.mid",
        None,
    )
    .unwrap();

    let ppq = midi.ppq();
    let merged = pipe!(
        midi.iter_all_track_events_merged_batches()
        |>TimeCaster::<f64>::cast_event_delta()
        |>cancel_tempo_events(250000)
        |>scale_event_time(1.0 / ppq as f64)
        |>unwrap_items()
    );

    println!("Tracks: {}", midi.track_count());

    let start = Instant::now();
    let mut note_count = 0;
    for batch in merged {
        for e in batch.iter_events() {
            if let Event::NoteOn(_) = e.as_event() {
                note_count += 1;
            }
        }
    }

    println!("Note count: {}   \tTime: {:?}", note_count, start.elapsed());
    println!(
        "Notes per second: {}",
        note_count as f64 / start.elapsed().as_secs_f64()
    );
}
