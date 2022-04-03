#[macro_export]
macro_rules! grouped_multithreaded_merge {
    ($array:ident, $fn_2:ident, $fn_arr:ident) => {
        {
            let buffer_size = 1 << 20;
            if $array.len() == 0 {
                return threaded_buffer(std::iter::empty(), 1);
            }
            if $array.len() == 1 {
                return threaded_buffer($array.remove(0), buffer_size);
            }

            let depth = 2;

            let count = 1 << depth;

            let mut iterator_groups = Vec::new();

            for _ in 0..count {
                iterator_groups.push(Vec::new());
            }

            for (i, iter) in $array.into_iter().enumerate() {
                let i = i % count;
                iterator_groups[i].push(iter);
            }

            let mut iterator_groups = pipe!(
                iterator_groups.into_iter()
                .map(|g| pipe!(
                    g
                    |>$fn_arr()
                    |>threaded_buffer(buffer_size)
                ))
                |>to_vec()
            );

            let mut new_groups = Vec::new();
            while iterator_groups.len() > 1 {
                while iterator_groups.len() != 0 {
                    if iterator_groups.len() >= 2 {
                        let merge = $fn_2(iterator_groups.remove(0), iterator_groups.remove(0));
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
    };
}
