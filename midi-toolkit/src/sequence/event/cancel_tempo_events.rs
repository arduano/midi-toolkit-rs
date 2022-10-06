use gen_iter::GenIter;

use crate::{
    events::{BatchTempo, MIDIDelta},
    num::MIDINum,
    unwrap,
};

pub fn cancel_tempo_events<
    D: MIDINum,
    E: BatchTempo + MIDIDelta<D>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    iter: I,
    new_tempo: u32,
) -> impl Iterator<Item = Result<E, Err>> {
    GenIter(move || {
        let zero = D::zero();
        let mut extra_ticks = zero;
        let mut tempo = D::midi_num_from(500000);
        let mut last_diff = zero;

        let new_tempo = tempo * (tempo / D::midi_num_from(new_tempo));

        for e in iter {
            let mut e = unwrap!(e);
            e.set_delta(e.delta() / new_tempo * tempo + extra_ticks);
            extra_ticks = zero;
            if let Some(inner_tempo) = e.inner_tempo() {
                tempo = D::midi_num_from(inner_tempo);
                extra_ticks = e.delta() + last_diff;
                last_diff = zero;
                if let Some(without_tempo) = e.without_tempo() {
                    e = without_tempo;
                } else {
                    continue;
                }
            }
            yield Ok(e);
        }
    })
}
