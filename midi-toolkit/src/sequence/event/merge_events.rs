use std::iter;

use gen_iter::GenIter;

use crate::{
    events::MIDIEvent,
    num::MIDINum,
    pipe,
    sequence::{threaded_buffer, to_vec},
    unwrap, yield_error,
};

struct SeqTime<T: MIDINum, E: MIDIEvent<T>, Err, I: Iterator<Item = Result<E, Err>> + Sized> {
    iter: I,
    time: T,
    next: Option<E>,
}

/// Merge an array of event iterators together into one iterator.
pub fn merge_events_array<
    T: MIDINum,
    E: MIDIEvent<T>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    array: Vec<I>,
) -> impl Iterator<Item = Result<E, Err>> {
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

/// Merge a pair of two different event iterators together into one iterator.
pub fn merge_events<
    T: MIDINum,
    E: MIDIEvent<T>,
    Err,
    I1: Iterator<Item = Result<E, Err>> + Sized,
    I2: Iterator<Item = Result<E, Err>> + Sized,
>(
    iter1: I1,
    iter2: I2,
) -> impl Iterator<Item = Result<E, Err>> {
    fn seq_from_iter<
        T: MIDINum,
        E: MIDIEvent<T>,
        Err,
        I: Iterator<Item = Result<E, Err>> + Sized,
    >(
        mut iter: I,
    ) -> Result<SeqTime<T, E, Err, I>, Err> {
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
                    time: e.delta(),
                    next: Some(e),
                }),
            },
        }
    }

    fn move_next<T: MIDINum, E: MIDIEvent<T>, Err, I: Iterator<Item = Result<E, Err>> + Sized>(
        mut seq: &mut SeqTime<T, E, Err, I>,
    ) -> Result<(), Err> {
        let next = seq.iter.next();
        let next = match next {
            None => None,
            Some(e) => match e {
                Err(e) => return Err(e),
                Ok(e) => {
                    seq.time += e.delta();
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

        let mut time = T::zero();

        macro_rules! yield_event {
            ($ev:ident, $time:expr) => {
                $ev.set_delta($time - time);
                time = $time;
                yield Ok($ev);
            };
        }

        macro_rules! flush_seq_and_return {
            ($seq:ident) => {
                while let Some(mut ev) = $seq.next.take() {
                    yield_event!(ev, $seq.time);
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
                let mut ev = seq1.next.take().unwrap();
                yield_event!(ev, seq1.time);
                unwrap!(move_next(&mut seq1));
            } else {
                let mut ev = seq2.next.take().unwrap();
                yield_event!(ev, seq2.time);
                unwrap!(move_next(&mut seq2));
            }
        }
    })
}

/// Group tracks into separate threads and merge them together
pub fn grouped_multithreaded_merge_arrays<
    T: 'static + MIDINum,
    E: 'static + MIDIEvent<T> + Sync + Send,
    Err: 'static + Sync + Send,
    I: 'static + Iterator<Item = Result<E, Err>> + Sized + Sync + Send,
>(
    mut array: Vec<I>,
) -> impl Iterator<Item = Result<E, Err>> {
    let buffer_size = 1 << 20;
    if array.len() == 0 {
        return threaded_buffer(iter::empty(), 1);
    }
    if array.len() == 1 {
        return threaded_buffer(array.remove(0), buffer_size);
    }

    let depth = 2;

    let count = 1 << depth;

    let mut iterator_groups = Vec::new();

    for _ in 0..count {
        iterator_groups.push(Vec::new());
    }

    for (i, iter) in array.into_iter().enumerate() {
        let i = i % count;
        iterator_groups[i].push(iter);
    }

    let mut iterator_groups =
        pipe!(
            iterator_groups.into_iter()
            .map(|g| pipe!(
                g
                |>merge_events_array()
                |>threaded_buffer(buffer_size)
            ))
            |>to_vec()
        );

    let mut new_groups = Vec::new();
    while iterator_groups.len() > 1 {
        while iterator_groups.len() != 0 {
            if iterator_groups.len() >= 2 {
                let merge = merge_events(iterator_groups.remove(0), iterator_groups.remove(0));
                new_groups.push(threaded_buffer(merge, buffer_size));
            } else {
                new_groups.push(iterator_groups.remove(0));
            }
        }
        iterator_groups = new_groups;
        new_groups = Vec::new();
    }

    threaded_buffer(iterator_groups.remove(0), buffer_size)
}
