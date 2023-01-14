use gen_iter::GenIter;

use crate::{
    events::MIDIDelta,
    grouped_multithreaded_merge,
    num::MIDINum,
    pipe,
    sequence::{threaded_buffer, to_vec},
    unwrap, yield_error,
};

enum Sequence<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized> {
    HasNext {
        iter: I,
        next: E,
        _phantom: std::marker::PhantomData<D>,
    },
    Empty,
}

impl<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized>
    Sequence<D, E, Err, I>
{
    fn new(iter: I) -> Result<Self, Err> {
        let mut iter = iter;
        match iter.next() {
            Some(Ok(next)) => Ok(Sequence::HasNext {
                iter,
                next,
                _phantom: std::marker::PhantomData,
            }),
            Some(Err(e)) => Err(e),
            None => Ok(Sequence::Empty),
        }
    }

    fn next(&mut self) -> Result<Option<E>, Err> {
        match self {
            Sequence::HasNext { iter, next, .. } => match iter.next() {
                Some(Ok(mut iter_next)) => {
                    let new_time = next.delta().saturating_add(iter_next.delta());
                    iter_next.set_delta(new_time);
                    let old_next = std::mem::replace(next, iter_next);
                    Ok(Some(old_next))
                }
                Some(Err(e)) => Err(e),
                None => {
                    let old = std::mem::replace(self, Sequence::Empty);

                    let old_next = match old {
                        Sequence::HasNext { next, .. } => next,
                        _ => unreachable!(),
                    };

                    Ok(Some(old_next))
                }
            },
            Sequence::Empty => Ok(None),
        }
    }

    fn time(&self) -> Option<D> {
        match self {
            Sequence::HasNext { next, .. } => Some(next.delta()),
            Sequence::Empty => None,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Sequence::HasNext { .. } => false,
            Sequence::Empty => true,
        }
    }
}

struct BinaryTreeSequenceMerge<
    D: MIDINum,
    E: MIDIDelta<D>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
> {
    sequences: Vec<Sequence<D, E, Err, I>>,
    heap: Vec<Option<D>>,
}

impl<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized>
    BinaryTreeSequenceMerge<D, E, Err, I>
{
    fn new(iters: impl Iterator<Item = I>) -> Result<Self, Err> {
        let mut sequences = vec![];

        for iter in iters {
            let seq = Sequence::new(iter)?;
            if !seq.is_empty() {
                sequences.push(seq);
            }
        }

        if sequences.len() == 0 {
            sequences.push(Sequence::Empty);
        }

        let heap = vec![None; sequences.len() - 1];

        let mut tree = Self { heap, sequences };

        for i in (0..tree.heap.len()).rev() {
            tree.update_time_from_children_for(i);
        }

        Ok(tree)
    }

    fn get_time_for(&self, index: usize) -> Option<D> {
        if index >= self.heap.len() {
            let index = index - self.heap.len();
            self.sequences.get(index).and_then(|x| x.time())
        } else {
            self.heap.get(index).and_then(|x| *x)
        }
    }

    fn calculate_time_from_children_for(&self, index: usize) -> Option<D> {
        let left = index * 2 + 1;
        let right = index * 2 + 2;

        let left_time = self.get_time_for(left);
        let right_time = self.get_time_for(right);

        match (left_time, right_time) {
            (Some(left_time), Some(right_time)) => {
                if left_time < right_time {
                    Some(left_time)
                } else {
                    Some(right_time)
                }
            }
            (Some(left_time), None) => Some(left_time),
            (None, Some(right_time)) => Some(right_time),
            (None, None) => None,
        }
    }

    fn update_time_from_children_for(&mut self, index: usize) -> Option<D> {
        let time = self.calculate_time_from_children_for(index);
        self.heap[index] = time;
        time
    }

    fn propagate_time_change_from(&mut self, index: usize) {
        let mut index = index;
        while index > 0 {
            index = (index - 1) / 2;
            self.update_time_from_children_for(index);
        }
    }

    fn find_smallest_sequence_index(&self) -> Option<usize> {
        // Empty if the root time is None
        if self.get_time_for(0).is_none() {
            return None;
        }

        let mut index = 0;
        loop {
            if index >= self.heap.len() {
                return Some(index - self.heap.len());
            }

            let left = index * 2 + 1;
            let right = index * 2 + 2;

            let left_time = self.get_time_for(left);
            let right_time = self.get_time_for(right);

            let next_index = match (left_time, right_time) {
                (Some(left_time), Some(right_time)) => {
                    if left_time < right_time {
                        left
                    } else {
                        right
                    }
                }
                (Some(_), None) => left,
                (None, Some(_)) => right,
                (None, None) => unreachable!(),
            };

            index = next_index;
        }
    }

    fn next(&mut self) -> Result<Option<E>, Err> {
        let index = self.find_smallest_sequence_index();
        if let Some(index) = index {
            let sequence = &mut self.sequences[index];
            let item = sequence.next()?.unwrap();

            self.propagate_time_change_from(index + self.heap.len());

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    fn assert_all_empty(&self) {
        for seq in &self.sequences {
            debug_assert!(seq.is_empty());
        }
    }
}

/// Merge an array of event iterators together into one iterator.
pub fn merge_events_array<
    D: MIDINum,
    E: MIDIDelta<D>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
>(
    array: Vec<I>,
) -> impl Iterator<Item = Result<E, Err>> {
    GenIter(move || {
        let tree = BinaryTreeSequenceMerge::new(array.into_iter());
        match tree {
            Err(e) => yield_error!(Err(e)),
            Ok(mut tree) => {
                let mut time = D::zero();
                while let Some(mut e) = unwrap!(tree.next()) {
                    let new_time = e.delta();
                    e.set_delta(e.delta() - time);
                    time = new_time;
                    yield Ok(e);
                }
                tree.assert_all_empty();
            }
        }
    })
}

struct SeqTime<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized> {
    iter: I,
    time: D,
    next: Option<E>,
}

/// Merge a pair of two different event iterators together into one iterator.
pub fn merge_events<
    D: MIDINum,
    E: MIDIDelta<D>,
    Err,
    I1: Iterator<Item = Result<E, Err>> + Sized,
    I2: Iterator<Item = Result<E, Err>> + Sized,
>(
    iter1: I1,
    iter2: I2,
) -> impl Iterator<Item = Result<E, Err>> {
    fn seq_from_iter<
        D: MIDINum,
        E: MIDIDelta<D>,
        Err,
        I: Iterator<Item = Result<E, Err>> + Sized,
    >(
        mut iter: I,
    ) -> Result<SeqTime<D, E, Err, I>, Err> {
        let first = iter.next();
        match first {
            None => Ok(SeqTime {
                iter,
                time: D::zero(),
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

    fn move_next<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized>(
        mut seq: &mut SeqTime<D, E, Err, I>,
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

        let mut time = D::zero();

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
pub fn grouped_multithreaded_merge_event_arrays<
    D: 'static + MIDINum,
    E: 'static + MIDIDelta<D> + Sync + Send,
    Err: 'static + Sync + Send,
    I: 'static + Iterator<Item = Result<E, Err>> + Sized + Sync + Send,
>(
    mut array: Vec<I>,
) -> impl Iterator<Item = Result<E, Err>> {
    grouped_multithreaded_merge!(array, merge_events, merge_events_array)
}
