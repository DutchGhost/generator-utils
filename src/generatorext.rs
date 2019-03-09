use crate::{filter::Filter, map::Map, mapped::Mapped, take::Take, takewhile::TakeWhile};
use std::ops::Generator;
use std::pin::Pin;

use crate::iter::GenIter;

pub trait GeneratorExt: Generator {
    // Should be safe,
    // following the idea that this function consumes `self` (moves it),
    // but in order for Self (the generator) to be invalidated in this function,
    // some `unsafe {}` must have been used before this function is called.
    #[inline]
    fn into_iter(self) -> GenIter<Self>
    where
        Self: Sized,
    {
        unsafe { GenIter::new_unchecked(self) }
    }

    #[inline]
    fn by_ref(&mut self) -> &mut Self {
        self
    }

    /// Takes a closure and creates a generator that calls the closure on each yielded element.
    /// `.map_yield()` transforms one generator into another, by means of its argument: something that implements [`FnMut`]. It produces a new
    /// generator which calls this closure on each yielded element of the original generator.
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Yield) -> O,
    {
        Map::new(self, f)
    }

    /// Creates a generator which uses a closure to determine if an element should be yielded.
    ///
    /// The closure must return `true` or `false`. `filter_yield()` creates a generator which calls this closure on each yielded element.
    /// If the closure returns `true`, then the element is yielded. If the closure returns `false`, it will try again, and call the closure on the next element,
    /// seeing if it passes the test.
    fn filter<F>(self, pred: F) -> Filter<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Yield) -> bool,
    {
        Filter::new(self, pred)
    }

    fn take_while<F>(self, predicate: F) -> TakeWhile<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Yield) -> bool,
    {
        TakeWhile::new(self, predicate)
    }

    fn take(self, count: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take::new(self, count)
    }

    /// Takes a closure and creates a new generator as a result of the closure.
    /// `.mapped()` transforms one generator into another, by means of its argument: something that implements [`FnMut`]. It produces a new
    /// generator as a result of the closure.
    ///
    /// # Example
    /// ```
    /// #![feature(generators, generator_trait)]
    ///
    /// use generator_utils::GeneratorExt;
    /// use generator_utils::iter::GenIter;
    ///
    /// use std::ops::Generator;
    ///
    /// fn generator() -> impl Generator<Yield = i32, Return = ()> {
    ///     || {
    ///         for i in 0..5 {
    ///             yield i
    ///         }
    ///     }
    /// }
    ///
    /// let mapped = generator().mapped(move |gen| {
    ///     || {
    ///         for item in GenIter::new(gen) {
    ///             yield item * 2
    ///         }
    ///     }
    /// });
    ///
    /// let mut iter = GenIter::new(mapped);
    ///
    /// for (g, n) in iter.by_ref().zip((0..5).map(|n| n * 2)) {
    ///     assert_eq!(g, n);
    /// }
    ///
    /// assert!(iter.next().is_none())
    ///
    /// ```
    fn mapped<F, G>(self, mut f: F) -> Mapped<G>
    where
        Self: Sized,
        F: FnMut(Self) -> G,
        G: Generator,
    {
        Mapped::new(f(self))
    }

    fn fold_ret<B, F>(mut self, mut init: B, mut f: F) -> (B, Self::Return)
    where
        Self: Sized,
        F: FnMut(B, Self::Yield) -> B,
    {
        use std::ops::GeneratorState;

        loop {
            let pin = unsafe { Pin::new_unchecked(&mut self) };

            match pin.resume() {
                GeneratorState::Yielded(y) => {
                    init = f(init, y);
                }
                GeneratorState::Complete(r) => break (init, r),
            }
        }
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Yield) -> B,
    {
        self.fold_ret(init, f).0
    }
}

impl<G> GeneratorExt for G where G: Generator {}

pub trait PinGeneratorExt: Generator + Unpin {
    #[inline]
    fn iter(&mut self) -> GenIter<&mut Self> {
        GenIter::new(self)
    }

    #[inline]
    fn pin(&mut self) -> Pin<&mut Self> {
        Pin::new(self)
    }
}

impl<G> PinGeneratorExt for G where G: Generator + Unpin {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_mapped() {
        use crate::iter::GenIter;
        use std::ops::Generator;

        fn generator() -> impl Generator<Yield = i32, Return = ()> {
            || {
                for i in 0..5 {
                    yield i
                }
            }
        }

        let mapped = generator().mapped(|gen| {
            move || {
                for item in GenIter::new(gen) {
                    yield item * 2
                }
            }
        });

        let mut iter = GenIter::new(mapped);

        for (g, n) in iter.by_ref().zip((0..5).map(|n| n * 2)) {
            assert_eq!(g, n);
        }

        assert!(iter.next().is_none())
    }
}
