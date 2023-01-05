use gen_iter::GenIter;

use crate::{
    grouped_multithreaded_merge,
    notes::MIDINote,
    num::MIDINum,
    pipe,
    sequence::{threaded_buffer, to_vec},
    unwrap, yield_error,
};

struct SeqTime<T: MIDINum, N: MIDINote<T>, Err, I: Iterator<Item = Result<N, Err>> + Sized> {
    iter: I,
    time: T,
    next: Option<N>,
}

/// Merge an array of note iterators together into one iterator.
pub fn merge_notes_array<
    T: MIDINum,
    N: MIDINote<T>,
    Err,
    I: Iterator<Item = Result<N, Err>> + Sized,
>(
    array: Vec<I>,
) -> impl Iterator<Item = Result<N, Err>> {
    GenIter(move || {
        let mut sequences = Vec::new();
        for mut seq in array.into_iter() {
            let first = seq.next();
            match first {
                None => continue,
                Some(e) => match e {
                    Err(e) => yield_error!(Err(e)),
                    Ok(e) => {
                        let s = SeqTime {
                            time: e.start(),
                            next: Some(e),
                            iter: seq,
                        };
                        sequences.push(s);
                    }
                },
            }
        }

        while !sequences.is_empty() {
            let mut smallest_index = 0;
            let mut smallest_time = sequences[0].time;
            for (i, next) in sequences.iter().enumerate() {
                if next.time < smallest_time {
                    smallest_time = next.time;
                    smallest_index = i;
                }
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
                        let mut smallest = &mut sequences[smallest_index];
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

/// Merge a pair of two different event iterators together into one iterator.
pub fn merge_notes<
    T: MIDINum,
    N: MIDINote<T>,
    Err: 'static,
    I1: Iterator<Item = Result<N, Err>> + Sized,
    I2: Iterator<Item = Result<N, Err>> + Sized,
>(
    iter1: I1,
    iter2: I2,
) -> impl Iterator<Item = Result<N, Err>> {
    fn seq_from_iter<
        T: MIDINum,
        N: MIDINote<T>,
        Err,
        I: Iterator<Item = Result<N, Err>> + Sized,
    >(
        mut iter: I,
    ) -> Result<SeqTime<T, N, Err, I>, Err> {
        let first = iter.next();
        match first {
            None => Ok(SeqTime {
                iter,
                time: T::zero(),
                next: None,
            }),
            Some(e) => match e {
                Err(e) => Err(e),
                Ok(e) => Ok(SeqTime {
                    iter,
                    time: e.start(),
                    next: Some(e),
                }),
            },
        }
    }

    fn move_next<T: MIDINum, N: MIDINote<T>, Err, I: Iterator<Item = Result<N, Err>> + Sized>(
        mut seq: &mut SeqTime<T, N, Err, I>,
    ) -> Result<(), Err> {
        let next = seq.iter.next();
        let next = match next {
            None => None,
            Some(e) => match e {
                Err(e) => return Err(e),
                Ok(e) => {
                    seq.time = e.start();
                    Some(e)
                }
            },
        };
        seq.next = next;
        Ok(())
    }

    GenIter(move || {
        let mut seq1 = unwrap!(seq_from_iter(iter1));
        let mut seq2 = unwrap!(seq_from_iter(iter2));

        macro_rules! flush_seq_and_return {
            ($seq:ident) => {
                while let Some(ev) = $seq.next.take() {
                    yield Ok(ev);
                    unwrap!(move_next(&mut $seq));
                }
                return;
            };
        }

        loop {
            if seq1.next.is_none() {
                if seq2.next.is_none() {
                    break;
                } else {
                    flush_seq_and_return!(seq2);
                }
            }
            if seq2.next.is_none() {
                flush_seq_and_return!(seq1);
            }

            if seq1.time < seq2.time {
                let ev = seq1.next.take().unwrap();
                yield Ok(ev);
                unwrap!(move_next(&mut seq1));
            } else {
                let ev = seq2.next.take().unwrap();
                yield Ok(ev);
                unwrap!(move_next(&mut seq2));
            }
        }
    })
}

/// Group tracks into separate threads and merge them together
pub fn grouped_multithreaded_merge_note_arrays<
    T: 'static + MIDINum,
    N: 'static + MIDINote<T> + Sync + Send,
    Err: 'static + Sync + Send,
    I: 'static + Iterator<Item = Result<N, Err>> + Sized + Sync + Send,
>(
    mut array: Vec<I>,
) -> impl Iterator<Item = Result<N, Err>> {
    grouped_multithreaded_merge!(array, merge_notes, merge_notes_array)
}
