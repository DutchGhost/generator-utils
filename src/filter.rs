use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

pub struct Filter<G, F> {
    gen: G,
    predicate: F,
}

impl<G, F> Filter<G, F> {
    #[inline]
    pub(crate) fn new(gen: G, predicate: F) -> Self {
        Self { gen, predicate }
    }
}

impl<G: Generator, F> Generator for Filter<G, F>
where
    F: FnMut(&G::Yield) -> bool,
{
    type Yield = G::Yield;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        unsafe {
            let _self: &mut Self = self.get_unchecked_mut();

            loop {
                // We need to create a Pin on each Iteration,
                // generators .resume() consumes the Pin.
                let gen = Pin::new_unchecked(&mut _self.gen);

                match gen.resume() {
                    GeneratorState::Yielded(y) => {
                        if (_self.predicate)(&y) {
                            break GeneratorState::Yielded(y);
                        }
                    }
                    GeneratorState::Complete(r) => break GeneratorState::Complete(r),
                }
            }
        }
    }
}
