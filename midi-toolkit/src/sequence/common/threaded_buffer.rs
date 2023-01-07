use crossbeam_channel::{bounded, unbounded, IntoIter, Sender};
use gen_iter::GenIter;
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    thread,
};

use crate::sequence::common::to_vec;

pub struct ThreadBufferIter<T> {
    backward_tx: Sender<VecDeque<T>>,
    bufs_iter: IntoIter<VecDeque<T>>,
    current_buf: Option<VecDeque<T>>,
}

impl<T> Iterator for ThreadBufferIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        loop {
            if let Some(ref mut buf) = self.current_buf {
                match buf.pop_front() {
                    Some(item) => return Some(item),
                    None => {
                        self.backward_tx.send(self.current_buf.take().unwrap()).ok();
                        self.current_buf = self.bufs_iter.next();
                    }
                }
            } else {
                return None;
            }
        }
    }
}

pub fn threaded_buffer<
    T: 'static + Send + Sync,
    I: 'static + Iterator<Item = T> + Sized + Send + Sync,
>(
    iter: I,
    buffer_size: usize,
) -> ThreadBufferIter<T> {
    let (forward_tx, forward_rx) = unbounded::<VecDeque<T>>();
    let (backward_tx, backward_rx) = unbounded::<VecDeque<T>>();
    thread::spawn(move || {
        let mut iter = iter;
        let mut ended = false;
        for mut vector in backward_rx.into_iter() {
            if !ended {
                for _ in 0..buffer_size {
                    match iter.next() {
                        Some(item) => vector.push_back(item),
                        None => {
                            ended = true;
                            break;
                        }
                    }
                }
            }

            if !vector.is_empty() {
                forward_tx.send(vector).ok();
            } else {
                break;
            }
        }
    });

    for _ in 0..3 {
        backward_tx.send(VecDeque::with_capacity(buffer_size)).ok();
    }

    let mut bufs_iter = forward_rx.into_iter();
    let current_buf = bufs_iter.next();

    ThreadBufferIter {
        bufs_iter,
        current_buf,
        backward_tx,
    }
}

pub fn channels_into_threadpool<
    T: 'static + Send + Sync,
    E: 'static + Send + Sync,
    I: 'static + Iterator<Item = Result<T, E>> + Sized + Send + Sync,
>(
    iters: Vec<I>,
    buffer_size: usize,
) -> Vec<impl Iterator<Item = Result<T, E>>> {
    let buffer_count = 3;

    struct ReadCommand<T, E> {
        vector: VecDeque<Result<T, E>>,
        response_sender: Sender<VecDeque<Result<T, E>>>,
        iter_id: usize,
    }

    let (request_queue_sender, request_queue_receiver) = unbounded();

    let mut output_iters = Vec::new();

    for iter_id in 0..iters.len() {
        let (tx, rx) = bounded::<VecDeque<Result<T, E>>>(buffer_count);

        let sender = request_queue_sender.clone();

        for _ in 0..buffer_count {
            sender
                .send(ReadCommand {
                    vector: VecDeque::with_capacity(buffer_size),
                    response_sender: tx.clone(),
                    iter_id,
                })
                .ok();
        }

        output_iters.push(GenIter(move || {
            for mut received in rx.into_iter() {
                if received.is_empty() {
                    break;
                }

                while let Some(item) = received.pop_front() {
                    yield item;
                }

                sender
                    .send(ReadCommand {
                        vector: received,
                        response_sender: tx.clone(),
                        iter_id,
                    })
                    .ok();
            }
        }));
    }

    thread::spawn(move || {
        struct IterEnded<I> {
            iter: I,
            ended: bool,
        }
        let iters = to_vec(iters.into_iter().map(|i| {
            Arc::new(RwLock::new(IterEnded {
                iter: i,
                ended: false,
            }))
        }));
        for req in request_queue_receiver.into_iter() {
            let iter = iters[req.iter_id].clone();
            rayon::spawn_fifo(move || {
                let mut iter = iter.write().unwrap();
                let mut vector = req.vector;
                if !iter.ended {
                    for _ in 0..buffer_size {
                        match iter.iter.next() {
                            Some(Ok(item)) => vector.push_back(Ok(item)),
                            Some(Err(error)) => {
                                vector.push_back(Err(error));
                                iter.ended = true;
                                break;
                            }
                            None => {
                                iter.ended = true;
                                break;
                            }
                        }
                    }
                }

                req.response_sender.send(vector).ok();
            });
        }
    });

    output_iters
}
