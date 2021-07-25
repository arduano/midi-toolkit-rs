use events::{CastEventDelta, ChannelEvent, DeltaNum, MIDIEvent, KeyEvent};
use num_traits::Num;

use crate::events::NoteOnEvent;

mod events;

fn main() {
    let a = NoteOnEvent {
        delta: 1,
        channel: 1,
        key: 1,
        velocity: 1,
    };

    let b: NoteOnEvent<f64> = a.cast_delta();

    let bx: Box<dyn MIDIEvent<i32>> = Box::new(a);

    // println!("{:?}", a.key());
    // a.as_key_event();
}
