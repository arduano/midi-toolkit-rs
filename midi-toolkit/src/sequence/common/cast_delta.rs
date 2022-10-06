use crate::{
    events::{CastEventDelta, MIDIEvent},
    num::MIDINum,
};

pub struct TimeCaster<DT: MIDINum>(DT);

impl<DT: MIDINum> TimeCaster<DT> {
    pub fn cast_event_delta<
        E: CastEventDelta<DT>,
        Err,
        I: Iterator<Item = Result<E, Err>> + Sized,
    >(
        iter: I,
    ) -> impl Iterator<Item = Result<E::Output, Err>> {
        iter.map(move |e| {
            let e = e?;
            Ok(e.cast_delta())
        })
    }
}
