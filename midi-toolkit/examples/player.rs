use std::{
    thread,
    time::{Duration, Instant},
};

use kdmapi::KDMAPI;
use midi_toolkit::{
    events::MIDIEvent,
    io::MIDIFile,
    pipe,
    sequence::{
        event::{cancel_tempo_events, merge_events_array, scale_event_time},
        to_vec, unwrap_items, TimeCaster,
    },
};

fn main() {
    let midi = MIDIFile::open("D:/Midis/Forgiveness_REBORN_FINAL.mid", None).unwrap();
    let ppq = midi.ppq();
    let merged = pipe!(
        midi.iter_all_tracks()
        |>to_vec()
        |>merge_events_array()
        |>TimeCaster::<f64>::cast_event_delta()
        |>cancel_tempo_events(250000)
        |>scale_event_time(1.0 / ppq as f64)
        |>unwrap_items()
    );

    let kdmapi = KDMAPI.open_stream();

    let now = Instant::now();
    let mut time = 0.0;
    for e in merged {
        if e.delta() != 0.0 {
            time += e.delta();
            let diff = time - now.elapsed().as_secs_f64();
            if diff > 0.0 {
                thread::sleep(Duration::from_secs_f64(diff));
            }
        }

        if let Some(serialized) = e.as_u32() {
            kdmapi.send_direct_data(serialized);
        }
    }
}
