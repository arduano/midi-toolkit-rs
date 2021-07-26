use crate::num::{MIDINum, MIDINumInto};
use events::Event;
use events::{CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent, NoteOnEvent};

use num_traits::Num;

pub mod events;
pub mod notes;
pub mod num;

fn main() {
    let a = NoteOnEvent::new(1, 1, 1, 1);
    let b = a.as_event();

    let c: Event<f32> = b.cast_delta();

    // println!("{:?}", b.delta());
    // a.as_key_event();
}
