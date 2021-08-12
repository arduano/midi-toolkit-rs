use gen_iter::GenIter;

use crate::{events::MIDIEvent, num::MIDINum, unwrap, yield_error};

// pub fn merge_events_with<
//     T: MIDINum,
//     E: MIDIEvent<T>,
//     Err,
//     I: Iterator<Item = Result<E, Err>> + Sized,
// >(
//     iter: I,
//     from: T,
//     to: T,
// ) -> impl Iterator<Item = Result<E, Err>> {
//     todo!()
// }

pub fn merge_events_array<
    T: MIDINum,
    E: MIDIEvent<T>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    array: Vec<I>,
) -> impl Iterator<Item = Result<E, Err>> {
    struct SeqTime<T: MIDINum, E: MIDIEvent<T>, Err, I: Iterator<Item = Result<E, Err>> + Sized> {
        iter: I,
        time: T,
        next: Option<E>,
    }

    GenIter(move || {
        let mut seqences = Vec::new();
        for mut seq in array.into_iter() {
            let first = seq.next();
            match first {
                None => continue,
                Some(e) => match e {
                    Err(e) => yield_error!(Err(e)),
                    Ok(e) => {
                        let s = SeqTime {
                            time: e.delta(),
                            next: Some(e),
                            iter: seq,
                        };
                        seqences.push(s);
                    }
                },
            }
        }

        let mut time = T::zero();
        while seqences.len() > 0 {
            let len = seqences.len();
            let mut smallest_index = 0;
            let mut smallest_time = seqences[0].time;
            for i in 0..len {
                let next = &seqences[i];
                if next.time < smallest_time {
                    smallest_time = next.time;
                    smallest_index = i;
                }
            }
            loop {
                let (event, next) = {
                    let smallest = &mut seqences[smallest_index];

                    let mut event = smallest.next.take().unwrap();
                    let new_delta = smallest.time - time;
                    event.set_delta(new_delta);
                    time = smallest.time;

                    (event, smallest.iter.next())
                };
                yield Ok(event);
                match next {
                    None => {
                        seqences.remove(smallest_index);
                        break;
                    }
                    Some(next) => {
                        let next = unwrap!(next);
                        let mut smallest = &mut seqences[smallest_index];
                        smallest.time += next.delta();
                        smallest.next = Some(next);
                    }
                }
                if seqences[smallest_index].time != smallest_time {
                    break;
                }
            }
        }
    })
}
