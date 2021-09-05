use crossbeam_channel::{IntoIter, bounded};
use std::thread;

pub fn threaded_buffer<
    T: 'static + Send + Sync,
    I: 'static + Iterator<Item = T> + Sized + Send + Sync,
>(
    iter: I,
    max_buffer_size: usize,
) -> IntoIter<T> {
    let (tx, rx) = bounded(max_buffer_size);
    thread::spawn(move || {
        for item in iter {
            match tx.send(item) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
    });

    rx.into_iter()
}
