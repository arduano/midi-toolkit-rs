/// Wraps each item `T` into `Result<T, ()>`
///
/// Useful because all built in functions use Result as the item type for error handling
pub fn wrap_ok<T, I: Iterator<Item = T> + Sized>(iter: I) -> impl Iterator<Item = Result<T, ()>> {
    iter.map(|v| Ok(v))
}
