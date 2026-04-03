use std::{
    io::Cursor,
    time::{Duration, Instant},
};

use midi_toolkit::{
    events::Event,
    io::{MIDIFile, MIDIWriter},
    prelude::*,
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
    let filename = "D:/Midis/Ra Ra Rasputin Ultimate Black MIDI Final.mid";
    let repeats = 4;

    println!("Opening midi...");
    let file = MIDIFile::open(filename, None).unwrap();

    println!("Tracks: {}", file.track_count());

    let loaded_tracks = file
        .iter_all_tracks()
        .map(|t| t.collect_vec_result().unwrap())
        .collect::<Vec<_>>();

    let mut nc: u64 = 0;
    for track in loaded_tracks.iter() {
        for e in track {
            if let Event::NoteOn(_) = **e {
                nc += 1
            }
        }
    }
    println!("Note count: {nc}");

    do_run("Parse all tracks individually", repeats, || {
        for track in file.iter_all_tracks() {
            for _ in track {}
        }
    });
    do_run("Merge all tracks together while parsing", repeats, || {
        let merged = file.iter_all_tracks().merge_all();
        for _ in merged {}
    });
    do_run("Clone all events", repeats, || {
        let iters = loaded_tracks.iter().map(|t| t.iter().cloned());
        for track in iters {
            for _ in track {}
        }
    });
    do_run(
        "Clone all events, then wrap and unwrap them in Result",
        repeats,
        || {
            let iters = loaded_tracks
                .iter()
                .map(|t| t.iter().cloned().into_ok().unwrap_items());
            for track in iters {
                for _ in track {}
            }
        },
    );
    do_run("Merge all tracks together while cloning", repeats, || {
        let iters = loaded_tracks
            .iter()
            .map(|t| t.iter().cloned().into_ok())
            .collect::<Vec<_>>();
        let merged = iters.into_iter().merge_all();
        for _ in merged {}
    });
    do_run("Write each track while cloning", repeats, || {
        let output = Cursor::new(Vec::<u8>::new());
        let writer = MIDIWriter::new_from_stream(Box::new(output), file.ppq()).unwrap();

        let iters = loaded_tracks.iter().map(|t| t.iter().cloned());
        for track in iters {
            let mut track_writer = writer.open_next_track();
            for e in track {
                track_writer.write_event(e).unwrap();
            }
        }
    });
    do_run("Merge each track while cloning then write", repeats, || {
        let output = Cursor::new(Vec::<u8>::new());
        let writer = MIDIWriter::new_from_stream(Box::new(output), file.ppq()).unwrap();

        let iters = loaded_tracks
            .iter()
            .map(|t| t.iter().cloned().into_ok())
            .collect::<Vec<_>>();
        let merged = iters.into_iter().merge_all().unwrap_items();
        let mut track_writer = writer.open_next_track();
        for e in merged {
            track_writer.write_event(e).unwrap();
        }
    });
}
