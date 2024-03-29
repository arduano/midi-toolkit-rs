use std::time::Instant;

use midi_toolkit::{
    events::Event,
    io::MIDIFile,
    pipe,
    sequence::{event::merge_events_array, to_vec, unwrap_items},
};

pub fn main() {
    println!("Opening midi...");
    let file = MIDIFile::open("D:/Midis/tau2.5.9.mid", None).unwrap();
    println!("Parsing midi...");
    let now = Instant::now();
    let mut nc: u64 = 0;
    // for track in file.iter_all_tracks(true) {
    //     // let track = TimeCaster::<f64>::cast_event_delta(track);
    //     for e in pipe!(track|>unwrap_items())
    //     // for e in pipe!(track|>TimeCaster::<f64>::cast_event_delta()|>scale_event_time(10.0)|>unwrap_items())
    //     {
    //         match e {
    //             Event::NoteOn(_) => nc += 1,
    //             _ => {}
    //         }
    //     }
    // }
    let merged = pipe!(file.iter_all_tracks()|>to_vec()|>merge_events_array()|>unwrap_items());

    for e in merged {
        if let Event::NoteOn(_) = *e {
            nc += 1
        }
        let delta = e.delta;
        if delta > 184467440737 {
            dbg!(e);
        }
    }

    println!("Finished parsing midi, found {nc} notes");
    println!("Elapsed {:?}", now.elapsed());
    println!(
        "Notes/second {}",
        (nc as f64 / now.elapsed().as_secs_f64()).round()
    );
}
