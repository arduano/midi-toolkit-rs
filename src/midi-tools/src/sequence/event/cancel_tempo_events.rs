use gen_iter::GenIter;

use crate::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    num::MIDINum,
    unwrap,
};

pub fn cancel_tempo_events<
    T: MIDINum,
    E: MIDIEventEnum<T>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    iter: I,
    new_tempo: u32,
) -> impl Iterator<Item = Result<E, Err>> {
    GenIter(move || {
        let zero = T::zero();
        let mut extra_ticks = zero;
        let mut tempo = T::midi_num_from(500000);
        let mut last_diff = zero;

        let new_tempo = tempo * (tempo / T::midi_num_from(new_tempo));

        for e in iter {
            let mut e = unwrap!(e);
            e.set_delta(e.delta() / new_tempo * tempo + extra_ticks);
            extra_ticks = zero;
            match e.as_event() {
                Event::Tempo(e) => {
                    tempo = T::midi_num_from(e.tempo);
                    extra_ticks = e.delta() + last_diff;
                    last_diff = zero;
                    continue;
                }
                _ => {}
            };
            yield Ok(e);
        }
    })
}
