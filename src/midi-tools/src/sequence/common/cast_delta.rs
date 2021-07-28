use crate::{
    events::{CastEventDelta, MIDIEvent},
    num::MIDINum,
};

pub struct TimeCaster<DT: MIDINum>(DT);

impl<DT: MIDINum> TimeCaster<DT> {
    pub fn cast_event_delta<
        DE: MIDIEvent<DT>,
        E: CastEventDelta<DT, DE>,
        Err,
        I: Iterator<Item = Result<E, Err>> + Sized,
    >(
        iter: I,
    ) -> impl Iterator<Item = Result<DE, Err>> {
        iter.map(move |e| {
            let e = e?;
            Ok(e.cast_delta())
        })
    }
}
