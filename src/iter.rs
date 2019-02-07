use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

/// A wrapper struct around Generators,
/// providing a safe implementation of the [`Iterator`] trait.
pub struct GenIter<G>(Option<G>);

impl<G: Generator + Unpin> GenIter<G> {
    /// Creates a new `GenIter` instance from a generator.
    /// The returned instance can be iterated over,
    /// consuming the generator.
    #[inline]
    pub fn new(gen: G) -> Self {
        Self(Some(gen))
    }
}

impl<G: Generator + Unpin> Iterator for GenIter<G> {
    type Item = G::Yield;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Pin::new(self).next()
    }
}

impl<G: Generator> GenIter<G> {
    /// Creates a new `GenIter` instance from a generator.
    ///
    /// The returned instance can be iterated over,
    /// consuming the generator.
    ///
    /// # Safety
    /// This function is marked unsafe,
    /// because the caller must ensure the generator is in a valid state.
    /// A valid state means that the generator has not been moved ever since it's creation.
    #[inline]
    pub unsafe fn new_unchecked(gen: G) -> Self {
        Self(Some(gen))
    }

    /// Creates a new `GenIter` instance from a Pinned, Boxed generator.
    /// Te returned instance can be iterated over,
    /// consuming the generator.
    #[inline]
    pub fn pinned(gen: Pin<Box<G>>) -> GenIter<Pin<Box<G>>> {
        Self(Some(gen))
    }
}

impl<G: Generator> Iterator for Pin<&mut GenIter<G>> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        let this: Pin<&mut GenIter<G>> = self.as_mut();

        // This should be safe.
        // this Iterator implementation is on a Pin<&mut GenIter<G>> where G: Generator.
        // In order to acquire such a Pin<&mut GenIter<G>> if G does *NOT* implement Unpin,
        // the unsafe `new_unchecked` function from the Pin type must be used anyway.
        //
        // Note that if G: Unpin, the Iterator implementation of GenIter<G> itself is used,
        // which just creates a Pin safely, and then delegates to this implementation.
        let gen: Pin<&mut Option<G>> = unsafe { this.map_unchecked_mut(|geniter| &mut geniter.0) };

        let gen: Option<Pin<&mut G>> = Option::as_pin_mut(gen);

        match gen.map(Generator::resume) {
            Some(GeneratorState::Yielded(y)) => Some(y),
            Some(GeneratorState::Complete(_)) => {
                self.set(GenIter(None));
                None
            }
            None => None,
        }
    }
}

/// Creates a new instance of a [`crate::iter::GenIter`] with the provided generator `$x`.
/// # Examples
/// ```
/// #![feature(generators, generator_trait)]
///
/// extern crate generator_utils;
/// use generator_utils::gen_iter;
/// 
/// let mut iter = gen_iter! {
///     let x = 10;
///     let r = &x;
///
///     for i in 0..5u32 {
///         yield i * *r
///     }
/// };
/// ```
#[macro_export]
macro_rules! gen_iter {
    ($($x:tt)*) => {

        // Safe, the Generator is directly passed into new_unchecked,
        // so it has not been moved
        unsafe {
            $crate::iter::GenIter::new_unchecked(static || {
                $($x)*
            })
        }
    };
}

#[macro_export]
macro_rules! bind_iter {
    ($name:ident = || { $($x:tt)* }) => {
        let mut _iter = gen_iter!($($x)*);

        // Safe, we just created the GenIter struct,
        // and have not moved it.
        let $name = unsafe { Pin::new_unchecked(&mut _iter) };
    }
}

#[cfg(test)]
mod tests {
    use super::GenIter;

    #[test]
    fn iter_movable_generator() {
        let mut iter = GenIter::new(|| {
            for i in 0..5u32 {
                yield i
            }
        });

        for (n, i) in (0..5).zip(iter.by_ref()) {
            assert!(n == i);
        }

        // Assert no panic happens when we call next(), and the generator already has completed.
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_static_generator() {
        use std::pin::Pin;

        let mut iter = gen_iter! {
            let x = 10;
            let r = &x;

            for i in 0..5u32 {
                yield i * *r
            }
        };

        let mut iter = unsafe { Pin::new_unchecked(&mut iter) };

        for (n, i) in (0..5).map(|n| n * 10).zip(iter.by_ref()) {
            assert!(n == i);
        }

        // Assert no panic happens when we call next(), and the generator already has completed.
        assert!(iter.next().is_none())
    }

    #[test]
    fn iter_over_vec() {
        use std::pin::Pin;

        let mut vec = vec![1, 2, 3, 4, 5];

        bind_iter!(
            iterable = || {
                let v: &mut Vec<i32> = &mut vec;

                for item in v {
                    yield item
                }
            }
        );

        assert!(iterable.count() == 5);
    }

    #[test]
    fn ergo_pin() {
        use ergo_pin::{ergo_pin};
        use std::ops::Generator;

        fn foo<'a, T: Default>(v: &'a mut Vec<T>) -> GenIter<impl Generator<Yield = &'a mut T, Return = ()>> {
            gen_iter! {
                v.insert(0, Default::default());
                for x in v {
                    yield x;
                }
            }
        }

        let mut v = vec![1, 2, 3, 4, 5, 6, 7];

        #[ergo_pin] {
            let mut iter = pin!(foo(&mut v));
            
            for (x, n) in iter.by_ref().zip(0..=7) {
                assert_eq!(*x, n);
            }

            assert!(iter.next().is_none());
        }

        assert!(v.len() == 8);
    }
}
