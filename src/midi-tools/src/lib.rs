#![feature(generators)]
#![feature(conservative_impl_trait)]

use crate::num::{MIDINum, MIDINumInto};
use events::Event;
use events::{CastEventDelta, ChannelEvent, KeyEvent, MIDIEvent, NoteOnEvent};
use fn_chain::chain;

use gen_iter::GenIter;
use num_traits::Num;
use to_vec::ToVec;

pub mod events;
pub mod notes;
pub mod num;

#[macro_export]
macro_rules! pipe {
    ($var:tt |> $function: ident($($params: expr),*) $($calls:tt)*) => {
        pipe!({$function($var, $($params),*)} $($calls)*)
    };
    ($var:tt) => {
        $var
    };
}

struct EventSequence<T: MIDINum, I: Iterator<Item = Event<T>> + Sized>(I);

impl<T: MIDINum, I: Iterator<Item = Event<T>> + Sized> Iterator for EventSequence<T, I> {
    type Item = Event<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

trait IntoNode<T: MIDINum, I: Iterator<Item = Event<T>> + Sized> {
    fn into_node(self) -> EventSequence<T, I>;
}

impl<T: MIDINum, I: Iterator<Item = Event<T>> + Sized> IntoNode<T, I> for I {
    fn into_node(self) -> EventSequence<T, I> {
        EventSequence(self)
    }
}

impl<T: MIDINum, I: Iterator<Item = Event<T>> + Sized> EventSequence<T, I> {
    fn change_ppq(
        self,
        from: T,
        to: T,
    ) -> EventSequence<T, impl Iterator<Item = Event<T>> + Sized> {
        EventSequence(GenIter(move || {
            for mut e in self {
                let delta = e.delta_mut();
                *delta = *delta * to / from;
                yield e;
            }
        }))
    }
}

fn change_ppq<T: MIDINum, E: MIDIEvent<T>, I: Iterator<Item = E> + Sized>(
    iter: I,
    from: T,
    to: T,
) -> impl Iterator<Item = E> {
    GenIter(move || {
        for mut e in iter {
            let delta = e.delta_mut();
            *delta = *delta * to / from;
            yield e;
        }
    })
}

#[test]
fn delta_change() {
    let events = vec![
        Event::new_note_on_event(100.0f64, 0, 64, 127),
        Event::new_note_off_event(50.0f64, 0, 64),
        Event::new_note_on_event(30.0f64, 0, 64, 127),
        Event::new_note_off_event(80.0f64, 0, 64),
    ];

    let seq = EventSequence(events.into_iter());

    let changed = change_ppq(seq, 64.0, 96.0).to_vec();

    assert_eq!(
        changed,
        vec![
            Event::new_note_on_event(150.0f64, 0, 64, 127),
            Event::new_note_off_event(75.0f64, 0, 64),
            Event::new_note_on_event(45.0f64, 0, 64, 127),
            Event::new_note_off_event(120.0f64, 0, 64),
        ]
    )
}

#[test]
fn delta_change_ints() {
    let events = vec![
        Event::new_note_on_event(100, 0, 64, 127),
        Event::new_note_off_event(50, 0, 64),
        Event::new_note_on_event(30, 0, 64, 127),
        Event::new_note_off_event(80, 0, 64),
    ];

    let seq = EventSequence(events.into_iter());

    let changed = pipe!{
        seq
        |> change_ppq(64, 96)
        |> change_ppq(96, 96 * 2)
        |> change_ppq(96 * 2, 96)
    }.to_vec();

    // // test_chain! {
    // //     change_ppq(64, 96)
    // // };

    assert_eq!(
        changed,
        vec![
            Event::new_note_on_event(150, 0, 64, 127),
            Event::new_note_off_event(75, 0, 64),
            Event::new_note_on_event(45, 0, 64, 127),
            Event::new_note_off_event(120, 0, 64),
        ]
    )
}

fn test<T: MIDINum, I: Iterator<Item = Event<T>> + Sized>(
    ev: EventSequence<T, I>,
) -> EventSequence<T, impl Iterator<Item = Event<T>> + Sized> {
    return ev;
}

fn main() {
    let a = NoteOnEvent::new(1, 1, 1, 1);
    let b = a.as_event();

    let c: Event<f32> = b.cast_delta();

    let events = vec![b];

    let events = EventSequence(events.iter().cloned());

    // Vec::new().iter()

    // println!("{:?}", b.delta());
    // a.as_key_event();
}
