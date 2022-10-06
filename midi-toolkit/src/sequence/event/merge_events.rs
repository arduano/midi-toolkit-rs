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
                    let new_time = next.delta() + iter_next.delta();
                    iter_next.set_delta(new_time);
                    let old_next = std::mem::replace(next, iter_next);
                    Ok(Some(old_next))
                }
                Some(Err(e)) => return Err(e),
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

enum BinaryCellState<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized> {
    Sequence(Sequence<D, E, Err, I>),
    Item { item: Option<E> },
}

impl<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized>
    BinaryCellState<D, E, Err, I>
{
    /// Function to make building the tree easier
    fn inner_sequence_or_panic(&mut self) -> &mut Sequence<D, E, Err, I> {
        match self {
            BinaryCellState::Sequence(seq) => seq,
            _ => panic!("Expected Sequence"),
        }
    }

    fn time(&self) -> Option<D> {
        match self {
            BinaryCellState::Sequence(seq) => seq.time(),
            BinaryCellState::Item { item } => match item {
                Some(item) => Some(item.delta()),
                None => None,
            },
        }
    }
}

struct BinaryIteratorTree<
    D: MIDINum,
    E: MIDIDelta<D>,
    Err,
    I: Iterator<Item = Result<E, Err>> + Sized,
> {
    heap: Vec<BinaryCellState<D, E, Err, I>>,
}

impl<D: MIDINum, E: MIDIDelta<D>, Err, I: Iterator<Item = Result<E, Err>> + Sized>
    BinaryIteratorTree<D, E, Err, I>
{
    fn new(mut iters: impl Iterator<Item = I>) -> Result<Self, Err> {
        let first = if let Some(first) = iters.next() {
            Sequence::new(first)?
        } else {
            Sequence::Empty
        };

        let mut heap_vec = vec![BinaryCellState::Sequence(first)];

        let triangulate_child = |current: &mut BinaryCellState<D, E, Err, I>,
                                 child1: &mut Sequence<D, E, Err, I>|
         -> Result<Option<Sequence<D, E, Err, I>>, Err> {
            let current_seq = current.inner_sequence_or_panic();

            let child1_time = match child1.time() {
                Some(time) => time,
                None => return Ok(None),
            };
            let current_time = current_seq.time().expect("Current node shouldn't be empty");

            let next = if current_time <= child1_time {
                current_seq.next()?
            } else {
                child1.next()?
            };
            let next = next.expect("Can't be empty here");

            let new_parent = BinaryCellState::Item { item: Some(next) };

            let child2 = match std::mem::replace(current, new_parent) {
                BinaryCellState::Sequence(seq) => seq,
                _ => unreachable!(),
            };

            Ok(Some(child2))
        };

        let mut i = 0;
        for iter in iters {
            let mut new_sequence = Sequence::new(iter)?;

            let current = heap_vec[i].inner_sequence_or_panic();
            if current.is_empty() {
                *current = new_sequence;
                continue;
            }

            let next_child = triangulate_child(&mut heap_vec[i], &mut new_sequence)?;

            if let Some(next_child) = next_child {
                heap_vec.push(BinaryCellState::Sequence(new_sequence));
                heap_vec.push(BinaryCellState::Sequence(next_child));
                i += 1;
            }
        }

        Ok(Self { heap: heap_vec })
    }

    fn inner_get_time(&mut self, index: usize) -> Result<Option<D>, Err> {
        let current = match self.heap.get(index) {
            Some(current) => current,
            None => return Ok(None),
        };

        Ok(current.time())
    }

    fn inner_recursive_get_next(&mut self, index: usize) -> Result<Option<E>, Err> {
        let current = match self.heap.get_mut(index) {
            Some(current) => current,
            None => return Ok(None),
        };

        let item = match current {
            BinaryCellState::Sequence(seq) => return Ok(seq.next()?),

            // Item nodes are handled below
            BinaryCellState::Item { item } => item,
        };

        let inner_item = if let Some(item) = item.take() {
            item
        } else {
            return Ok(None);
        };

        let left_child_index = 2 * index + 1;
        let right_child_index = 2 * index + 2;

        let left_child_time = self.inner_get_time(left_child_index)?;
        let right_child_time = self.inner_get_time(right_child_index)?;

        let next = match (left_child_time, right_child_time) {
            (None, None) => {
                return Ok(Some(inner_item));
            }
            (Some(_), None) => self.inner_recursive_get_next(left_child_index)?.unwrap(),
            (None, Some(_)) => self.inner_recursive_get_next(right_child_index)?.unwrap(),
            (Some(left_child_time), Some(right_child_time)) => {
                if left_child_time <= right_child_time {
                    self.inner_recursive_get_next(left_child_index)?.unwrap()
                } else {
                    self.inner_recursive_get_next(right_child_index)?.unwrap()
                }
            }
        };

        // Need to get current again so that rust doesn't complain about mut reference
        match self.heap.get_mut(index).unwrap() {
            BinaryCellState::Sequence(_) => unreachable!(),
            BinaryCellState::Item { item } => *item = Some(next),
        };

        Ok(Some(inner_item))
    }

    fn next(&mut self) -> Result<Option<E>, Err> {
        self.inner_recursive_get_next(0)
    }

    fn assert_all_empty(&self) {
        for cell in &self.heap {
            match cell {
                BinaryCellState::Sequence(seq) => assert!(seq.is_empty()),
                BinaryCellState::Item { item } => assert!(item.is_none()),
            }
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
        let tree = BinaryIteratorTree::new(array.into_iter());
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
