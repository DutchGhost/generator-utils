use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

pub struct Mapped<G>(G);

impl<G> Mapped<G> {
    #[inline]
    pub(crate) fn new(generator: G) -> Self {
        Self(generator)
    }
}

impl<G: Generator> Generator for Mapped<G> {
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        unsafe { self.map_unchecked_mut(|gen| &mut gen.0).resume() }
    }
}
