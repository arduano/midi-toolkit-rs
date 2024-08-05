use core::iter::{FusedIterator, Iterator};
use core::marker::Unpin;
use core::ops::{Coroutine, CoroutineState};
use core::pin::Pin;

/// an iterator that holds an internal coroutine representing
/// the iteration state
#[derive(Copy, Clone, Debug)]
pub struct GenIter<T>(pub T)
where
    T: Coroutine<Return = ()> + Unpin;

impl<T> Iterator for GenIter<T>
where
    T: Coroutine<Return = ()> + Unpin,
{
    type Item = T::Yield;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match Pin::new(&mut self.0).resume(()) {
            CoroutineState::Yielded(n) => Some(n),
            CoroutineState::Complete(()) => None,
        }
    }
}

impl<G> From<G> for GenIter<G>
where
    G: Coroutine<Return = ()> + Unpin,
{
    #[inline]
    fn from(gen: G) -> Self {
        GenIter(gen)
    }
}

/// macro to simplify iterator - via - coroutine construction
///
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::gen_iter;
///
/// let mut g = gen_iter!({
///     yield 1;
///     yield 2;
/// });
///
/// assert_eq!(g.next(), Some(1));
/// assert_eq!(g.next(), Some(2));
/// assert_eq!(g.next(), None);
///
/// ```
#[macro_export]
macro_rules! gen_iter {
    ($block: block) => {
        $crate::gen_iter::GenIter(
            #[coroutine]
            || $block,
        )
    };
    (move $block: block) => {
        $crate::gen_iter::GenIter(
            #[coroutine]
            move || $block,
        )
    };
}

/// `GenIterReturn<G>` holds a coroutine `G` or the return value of `G`,
/// `&mut GenIterReturn<G>` acts as an iterator.
///
/// Differences with `GenIter<G>`:
/// 1. able to get return value of a coroutine
/// 2. safe to call `next()` after coroutine is done without panic
/// 3. maybe less efficient than `GenIter<G>`
#[derive(Copy, Clone, Debug)]
pub struct GenIterReturn<G: Coroutine + Unpin>(Result<G::Return, G>);

impl<G: Coroutine + Unpin> GenIterReturn<G> {
    #[inline]
    pub fn new(g: G) -> Self {
        GenIterReturn(Err(g))
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.0.is_ok()
    }

    #[inline]
    pub fn return_or_self(self) -> Result<G::Return, Self> {
        match self.0 {
            Ok(r) => Ok(r),
            Err(_) => Err(self),
        }
    }
}

/// Force use `&mut g` as iterator to prevent the code below,
/// in which return value cannot be got.
/// ```compile_fail
/// // !!INVALID CODE!!
/// # #![feature(coroutines)]
/// # use gen_iter::gen_iter_return;
/// let mut g = gen_iter_return!({ yield 1; return "done"; });
/// for v in g {} // invalid, because `GenIterReturn<G>` is not `Iterator`
/// let ret = g.return_or_self(); // g is dropped after for loop
/// ```
impl<G: Coroutine + Unpin> Iterator for &mut GenIterReturn<G> {
    type Item = G::Yield;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Ok(_) => None,
            Err(ref mut g) => match Pin::new(g).resume(()) {
                CoroutineState::Yielded(y) => Some(y),
                CoroutineState::Complete(r) => {
                    self.0 = Ok(r);
                    None
                }
            },
        }
    }
}

/// `GenIterReturn<G>` satisfies the trait `FusedIterator`
impl<G: Coroutine + Unpin> FusedIterator for &mut GenIterReturn<G> {}

impl<G: Coroutine + Unpin> From<G> for GenIterReturn<G> {
    #[inline]
    fn from(g: G) -> Self {
        GenIterReturn::new(g)
    }
}

/// macro to simplify iterator - via - coroutine with return value construction
/// ```
/// #![feature(coroutines)]
///
/// use gen_iter::gen_iter_return;
///
/// let mut g = gen_iter_return!({
///     yield 1;
///     yield 2;
///     return "done";
/// });
///
/// assert_eq!((&mut g).collect::<Vec<_>>(), [1, 2]); // use `&mut g` as an iterator
/// assert_eq!(g.is_done(), true); // check whether coroutine is done
/// assert_eq!((&mut g).next(), None); // safe to call `next()` after done
/// assert_eq!(g.return_or_self().ok(), Some("done")); // get return value of coroutine
/// ```
#[macro_export]
macro_rules! gen_iter_return {
    ($block: block) => {
        $crate::gen_iter::GenIterReturn::new(
            #[coroutine]
            || $block,
        )
    };
    (move $block: block) => {
        $crate::gen_iter::GenIterReturn::new(
            #[coroutine]
            move || $block,
        )
    };
}

#[cfg(test)]
mod tests {
    use super::GenIterReturn;

    #[test]
    fn gen_iter_works() {
        let mut g = gen_iter!({
            yield 1;
            yield 2;
        });

        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    #[test]
    fn gen_iter_macro() {
        let mut g = gen_iter!(move {
            yield 1;
            yield 2;
        });

        assert_eq!(g.next(), Some(1));
        assert_eq!(g.next(), Some(2));
        assert_eq!(g.next(), None);
    }

    /// test `new` and all instance method,
    /// and show that it won't panic when call `next()` even exhausted.
    #[test]
    fn gen_iter_return_works() {
        let mut g = GenIterReturn::new(
            #[coroutine]
            || {
                yield 1;
                "done"
            },
        );

        assert_eq!((&mut g).next(), Some(1));
        assert!(!g.is_done());

        g = match g.return_or_self() {
            Ok(_) => panic!("coroutine is done but should not"),
            Err(g) => g,
        };

        assert_eq!((&mut g).next(), None);
        assert!(g.is_done());

        assert_eq!((&mut g).next(), None); // it won't panic when call `next()` even exhausted.

        assert_eq!(g.return_or_self().ok(), Some("done"));
    }

    #[test]
    fn from_coroutine() {
        let mut g = GenIterReturn::from(
            #[coroutine]
            || {
                yield 1;
                "done"
            },
        );

        assert_eq!((&mut g).next(), Some(1));
        assert_eq!((&mut g).next(), None);

        assert!(g.is_done());
        assert_eq!(g.return_or_self().ok(), Some("done"));
    }

    /// normal usage using macro `gen_iter_return`
    #[test]
    fn macro_usage() {
        let mut g = gen_iter_return!(move {
            yield 1;
            yield 2;
            return "done";
        });

        let (mut sum, mut count) = (0, 0);
        for y in &mut g {
            sum += y;
            count += 1;
        }
        assert_eq!((sum, count), (3, 2));

        assert!(g.is_done());
        assert_eq!(g.return_or_self().ok(), Some("done"));
    }
}
