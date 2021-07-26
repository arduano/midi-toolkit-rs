use events::Event;
use events::{
    CastEventDelta, ChannelEvent, MIDINum, MIDINumInto, KeyEvent, MIDIEvent, NoteOnEvent,
};
use num_traits::Num;

pub mod events;
pub mod notes;

fn main() {
    let a = NoteOnEvent::new(1, 1, 1, 1);
    let b = a.as_event();

    let c: Event<f32> = b.cast_delta();

    // println!("{:?}", b.delta());
    // a.as_key_event();
}
