use std::time::{Duration, Instant};

use midi_toolkit::{
    events::{Event, MIDIEventEnum},
    io::MIDIFile,
    pipe,
    sequence::{
        event::{cancel_tempo_events, get_channels_array_statistics, scale_event_time},
        unwrap_items, TimeCaster,
    },
};

fn do_run(name: &str, repeats: i32, run: impl Fn() -> u64) {
    let mut times = Vec::new();
    let mut note_count = 0;
    for _ in 0..repeats {
        let start = Instant::now();
        note_count = run();
        times.push(start.elapsed());
    }

    let mean = times.iter().map(|t| t.as_secs_f64()).sum::<f64>() / repeats as f64;

    println!(
        "Repeats: {}   \tMin: {:?}   \tMax: {:?}   \tAvg: {:?}   \tAvg NPS: {:?}   \tName: {}",
        repeats,
        times.iter().min().unwrap(),
        times.iter().max().unwrap(),
        Duration::from_secs_f64(mean),
        note_count as f64 / mean,
        name
    );
}

fn main() {
    let filename = "F:/Fast MIDIs/The Nuker 3 F3.mid";
    let repeats = 4;

    println!("Opening midi...");
    let file = MIDIFile::open(filename, None).unwrap();

    println!("Tracks: {}", file.track_count());

    // Make windows cache stuff
    let tracks = file.iter_all_tracks().collect();
    let stats = get_channels_array_statistics(tracks).unwrap();

    println!("Note count: {}", stats.note_count());

    do_run("Parse tracks in parallel", repeats, || {
        let tracks = file.iter_all_tracks().collect();
        let stats = get_channels_array_statistics(tracks).unwrap();
        return stats.note_count();
    });

    do_run("Iter event batches merged", repeats, || {
        let ppq = file.ppq();
        let merged = pipe!(
            file.iter_all_track_events_merged_batches()
            |>TimeCaster::<f64>::cast_event_delta()
            |>cancel_tempo_events(250000)
            |>scale_event_time(1.0 / ppq as f64)
            |>unwrap_items()
        );

        let mut note_count = 0;
        for batch in merged {
            for e in batch.into_iter() {
                match e.as_event() {
                    Event::NoteOn(_) => note_count += 1,
                    _ => {}
                }
            }
        }

        return note_count;
    });

    // do_run("Merge all tracks together while parsing", repeats, || {
    //     let merged = pipe!(file.iter_all_tracks()|>to_vec()|>merge_events_array());
    //     for _ in merged {}
    // });
    // do_run("Clone all events", repeats, || {
    //     let iters = pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned())));
    //     for track in iters {
    //         for _ in track {}
    //     }
    // });
    // do_run(
    //     "Clone all events, then wrap and unwrap them in Result",
    //     repeats,
    //     || {
    //         let iters = pipe!(loaded_tracks
    //             .iter()
    //             .map(|t| pipe!(t.iter().cloned()|>wrap_ok()|>unwrap_items())));
    //         for track in iters {
    //             for _ in track {}
    //         }
    //     },
    // );
    // do_run("Merge all tracks together while cloning", repeats, || {
    //     let iters =
    //         pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned()|>wrap_ok()))|>to_vec());
    //     let merged = pipe!(iters|>merge_events_array());
    //     for _ in merged {}
    // });
    // do_run("Write each track while cloning", repeats, || {
    //     let output = Cursor::new(Vec::<u8>::new());
    //     let writer = MIDIWriter::new_from_stram(Box::new(output), file.ppq()).unwrap();

    //     let iters = pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned())));
    //     for track in iters {
    //         let mut track_writer = writer.open_next_track();
    //         for e in track {
    //             track_writer.write_event(e).unwrap();
    //         }
    //     }
    // });
    // do_run("Merge each track while cloning then write", repeats, || {
    //     let output = Cursor::new(Vec::<u8>::new());
    //     let writer = MIDIWriter::new_from_stram(Box::new(output), file.ppq()).unwrap();

    //     let iters =
    //         pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned()|>wrap_ok()))|>to_vec());
    //     let merged = pipe!(iters|>merge_events_array()|>unwrap_items());
    //     let mut track_writer = writer.open_next_track();
    //     for e in merged {
    //         track_writer.write_event(e).unwrap();
    //     }
    // });
}
