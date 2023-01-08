use std::time::{Duration, Instant};

use midi_toolkit::{
    io::MIDIFile,
    pipe,
    sequence::{
        event::{get_channel_statistics, get_channels_array_statistics, merge_events_array},
        to_vec,
    },
};

fn duration_to_minutes_seconds(duration: Duration) -> String {
    format!(
        "{:02}:{:02}",
        duration.as_secs() / 60,
        duration.as_secs() % 60
    )
}

fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open("/run/media/d/Midis/Po Pi Po V4.mid", None).unwrap();
    println!("Parsing midi...");

    let now = Instant::now();
    let stats1 = pipe!(
        file.iter_all_events_merged()
        |>get_channel_statistics().unwrap()
    );

    println!("Calculated merged stats in {:?}", now.elapsed());
    println!(
        "MIDI length: {}",
        duration_to_minutes_seconds(stats1.calculate_total_duration(file.ppq()))
    );
    println!("Other stats: {stats1:#?}\n\n");

    let now = Instant::now();
    let stats2 = pipe!(
        file.iter_all_tracks()|>to_vec()|>get_channels_array_statistics().unwrap()
    );
    println!("Calculated multithreaded stats in {:?}", now.elapsed());
    println!(
        "MIDI length: {}",
        duration_to_minutes_seconds(stats2.calculate_total_duration(file.ppq()))
    );
    println!("Other stats: {stats2:#?}");
}
