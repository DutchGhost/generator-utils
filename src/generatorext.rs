use std::{
    pin::Pin,
    ops::{Generator, GeneratorState}
};

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

    fn take_while_yield<F>(self, predicate: F) -> TakeWhileYield<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Yield) -> bool
    {
        TakeWhileYield::new(self, predicate)
    }
}

impl <G> GeneratorExt for G where G: Generator {}

pub struct TakeWhileYield<G, P> {
    complete: bool,
    gen: G,
    predicate: P,
}

impl <G, P> TakeWhileYield<G, P> {
    pub(crate) fn new(gen: G, predicate: P) -> Self {
        Self { gen, predicate, complete: false }
    }
}

impl <G: Generator, P> Generator for TakeWhileYield<G, P>
where
    P: FnMut(&G::Yield) -> bool
{
    type Yield = G::Yield;
    type Return = ();

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        if self.complete {
            return GeneratorState::Complete(())
        }

        unsafe {
            let _self: &mut Self = self.get_unchecked_mut();

            match Pin::new_unchecked(&mut _self.gen).resume() {
                GeneratorState::Yielded(y) => {
                    if (_self.predicate)(&y) {
                        GeneratorState::Yielded(y)
                    } else {
                        _self.complete = true;
                        GeneratorState::Complete(())
                    }
                }
                _ => GeneratorState::Complete(())
            }
        }
    }
}