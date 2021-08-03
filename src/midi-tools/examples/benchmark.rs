use std::{
    io::Cursor,
    time::{Duration, Instant},
};

use midi_tools::{
    events::Event,
    io::{midi_file::MIDIFile, midi_writer::MIDIWriter},
    pipe,
    sequence::{event::merge_events_array, to_vec, to_vec_result, unwrap_items, wrap_ok},
};

fn do_run<T: Fn()>(name: &str, repeats: i32, run: T) {
    let mut times = Vec::new();
    for _ in 0..repeats {
        let start = Instant::now();
        run();
        times.push(start.elapsed());
    }

    let mean = times.iter().map(|t| t.as_secs_f64()).sum::<f64>() / repeats as f64;

    println!(
        "Repeats: {}   \tMin: {:?}   \tMax: {:?}   \tAvg: {:?}   \tName: {}",
        repeats,
        times.iter().min().unwrap(),
        times.iter().max().unwrap(),
        Duration::from_secs_f64(mean),
        name
    );
}

fn main() {
    let filename = "D:/Midis/tau2.5.9.mid";
    let repeats = 1;

    println!("Opening midi...");
    let file = MIDIFile::open(filename, None).unwrap();

    println!("Tracks: {}", file.track_count());

    let loaded_tracks = to_vec(file.iter_all_tracks().map(|t| to_vec_result(t).unwrap()));

    let mut nc: u64 = 0;
    for track in loaded_tracks.iter() {
        for e in track {
            match e {
                Event::NoteOn(_) => nc += 1,
                _ => {}
            }
        }
    }
    println!("Note count: {}", nc);

    do_run("Parse all tracks individually", repeats, || {
        for track in file.iter_all_tracks() {
            for _ in pipe!(track) {}
        }
    });
    do_run("Merge all tracks together while parsing", repeats, || {
        let merged = pipe!(file.iter_all_tracks()|>to_vec()|>merge_events_array());
        for _ in merged {}
    });
    do_run("Clone all events", repeats, || {
        let iters = pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned())));
        for track in iters {
            for _ in track {}
        }
    });
    do_run(
        "Clone all events, then wrap and unwrap them in Result",
        repeats,
        || {
            let iters = pipe!(loaded_tracks
                .iter()
                .map(|t| pipe!(t.iter().cloned()|>wrap_ok()|>unwrap_items())));
            for track in iters {
                for _ in track {}
            }
        },
    );
    do_run("Merge all tracks together while cloning", repeats, || {
        let iters =
            pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned()|>wrap_ok()))|>to_vec());
        let merged = pipe!(iters|>merge_events_array());
        for _ in merged {}
    });
    do_run("Write each track while cloning", repeats, || {
        let output = Cursor::new(Vec::<u8>::new());
        let writer = MIDIWriter::new_from_stram(Box::new(output), file.ppq()).unwrap();

        let iters = pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned())));
        for track in iters {
            let mut track_writer = writer.open_next_track();
            for e in track {
                track_writer.write_event(e).unwrap();
            }
        }
    });
    do_run("Merge each track while cloning then write", repeats, || {
        let output = Cursor::new(Vec::<u8>::new());
        let writer = MIDIWriter::new_from_stram(Box::new(output), file.ppq()).unwrap();

        let iters =
            pipe!(loaded_tracks.iter().map(|t| pipe!(t.iter().cloned()|>wrap_ok()))|>to_vec());
        let merged = pipe!(iters|>merge_events_array()|>unwrap_items());
        let mut track_writer = writer.open_next_track();
        for e in merged {
            track_writer.write_event(e).unwrap();
        }
    });
}
