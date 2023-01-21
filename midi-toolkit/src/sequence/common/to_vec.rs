use std::iter::FromIterator;

/// Converts an iterator into a vector.
///
/// Useful when you to cache the result of an iterator for future use.
///
/// Unwraps all results from the iterator items.
pub fn to_vec<T, I: Iterator<Item = T> + Sized>(iter: I) -> Vec<T> {
    FromIterator::from_iter(iter)
}

/// Converts a result iterator into a vector result.
///
/// Useful when you to cache the result of an iterator for future use.
///
/// Unwraps all results from the iterator items.
pub fn to_vec_result<T, Err, I: Iterator<Item = Result<T, Err>> + Sized>(
    iter: I,
) -> Result<Vec<T>, Err> {
    FromIterator::from_iter(iter)
}
