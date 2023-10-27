use crate::pipe;

use super::{threaded_buffer, to_vec};

pub trait MergableStreams {
    type Item: Send + Sync + 'static;

    fn merge_two(
        iter1: impl Iterator<Item = Self::Item> + Send + Sync + 'static,
        iter2: impl Iterator<Item = Self::Item> + Send + Sync + 'static,
    ) -> impl Iterator<Item = Self::Item> + Send + Sync + 'static;

    fn merge_array(
        array: Vec<impl Iterator<Item = Self::Item> + Send + Sync + 'static>,
    ) -> impl Iterator<Item = Self::Item> + Send + Sync + 'static;
}

pub fn grouped_multithreaded_merge<T: MergableStreams>(
    mut array: Vec<impl Iterator<Item = T::Item> + Send + Sync + 'static>,
) -> impl Iterator<Item = T::Item> {
    {
        let buffer_size = 1 << 20;
        if array.is_empty() {
            return threaded_buffer(std::iter::empty(), 1);
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

        let mut iterator_groups = pipe!(
            iterator_groups.into_iter()
            .map(|g| pipe!(
                g
                |>T::merge_array()
                |>threaded_buffer(buffer_size)
            ))
            |>to_vec()
        );

        let mut new_groups = Vec::new();
        while iterator_groups.len() > 1 {
            while !iterator_groups.is_empty() {
                if iterator_groups.len() >= 2 {
                    let merge = T::merge_two(iterator_groups.remove(0), iterator_groups.remove(0));
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
}
