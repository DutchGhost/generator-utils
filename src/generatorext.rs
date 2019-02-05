use std::ops::{Generator, GeneratorState};

use crate::{
    mapyield::MapYield,
    filteryield::FilterYield,
};

pub trait GeneratorExt: Generator {

    /// Takes a closure and creates a generator that calls the closure on each yielded element.
    /// `.map_yield()` transforms one generator into another, by means of its argument: something that implements [`FnMut`]. It produces a new
    /// generator which calls this closure on each yielded element of the original generator.
    fn map_yield<F, O>(self, f: F) -> MapYield<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Yield) -> O
    {
        MapYield::new(self, f)
    }

    /// Creates a generator which uses a closure to determine if an element should be yielded.
    /// 
    /// The closure must return `true` or `false`. `filter_yield()` creates a generator which calls this closure on each yielded element.
    /// If the closure returns `true`, then the element is yielded. If the closure returns `false`, it will try again, and call the closure on the next element,
    /// seeing if it passes the test.
    fn filter_yield<F>(self, pred: F) -> FilterYield<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Yield) -> bool
    {
        FilterYield::new(self, pred)
    }
}

impl <G> GeneratorExt for G where G: Generator {}