use gen_iter::GenIter;

use crate::{notes::MIDINote, num::MIDINum, unwrap};

/// Merge an array of note iterators together into one iterator.
pub fn merge_notes_iterator<
    T: MIDINum,
    N: MIDINote<T>,
    Err,
    I: Iterator<Item = Result<N, Err>> + Sized,
    II: Iterator<Item = I> + Sized,
>(
    mut iter: II,
) -> impl Iterator<Item = Result<N, Err>> {
    struct SeqTime<T: MIDINum, N: MIDINote<T>, Err, I: Iterator<Item = Result<N, Err>> + Sized> {
        iter: I,
        time: T,
        next: Option<N>,
    }

    GenIter(move || {
        let mut get_next_seq = move || {
            for mut seq in iter.by_ref() {
                let first = seq.next();
                match first {
                    None => continue,
                    Some(e) => match e {
                        Err(e) => return Err(e),
                        Ok(e) => {
                            let s = SeqTime {
                                time: e.start(),
                                next: Some(e),
                                iter: seq,
                            };

                            return Ok(Some(s));
                        }
                    },
                }
            }
            Ok(None)
        };

        let mut sequences = Vec::new();

        let mut next_seq = unwrap!(get_next_seq());

        loop {
            if sequences.is_empty() {
                if let Some(next) = next_seq.take() {
                    sequences.push(next);
                    next_seq = unwrap!(get_next_seq());
                } else {
                    break;
                }
            }

            let mut smallest_index = 0;
            let mut smallest_time = sequences[0].time;
            for (i, next) in sequences.iter().enumerate() {
                if next.time < smallest_time {
                    smallest_time = next.time;
                    smallest_index = i;
                }
            }

            let is_next_seq_earlier = match next_seq {
                None => false,
                Some(ref next) => next.time < smallest_time,
            };

            if is_next_seq_earlier {
                sequences.push(next_seq.take().unwrap());
                next_seq = unwrap!(get_next_seq());
                continue;
            }

            loop {
                let (note, next) = {
                    let smallest = &mut sequences[smallest_index];

                    let note = smallest.next.take().unwrap();

                    (note, smallest.iter.next())
                };
                yield Ok(note);
                match next {
                    None => {
                        sequences.remove(smallest_index);
                        break;
                    }
                    Some(next) => {
                        let next = unwrap!(next);
                        let smallest = &mut sequences[smallest_index];
                        smallest.time = next.start();
                        smallest.next = Some(next);
                    }
                }
                if sequences[smallest_index].time != smallest_time {
                    break;
                }
            }
        }
    })
}
