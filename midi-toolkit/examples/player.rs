#[cfg(all(feature = "player-example", target_os = "windows"))]
mod player_impl {
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

    pub fn main() {
        let midi_path = std::env::args()
            .nth(1)
            .expect("usage: cargo run --example player --features player-example -- <midi-file>");

        let midi = MIDIFile::open(midi_path, None).unwrap();
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
            if e.delta != 0.0 {
                time += e.delta;
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
}

#[cfg(all(feature = "player-example", target_os = "windows"))]
fn main() {
    player_impl::main();
}

#[cfg(not(all(feature = "player-example", target_os = "windows")))]
fn main() {
    eprintln!(
        "The player example is only available on Windows with the `player-example` feature enabled."
    );
    eprintln!("It also requires the external `kdmapi` backend at runtime.");
}
