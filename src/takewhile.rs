use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

pub struct TakeWhile<G, P> {
    complete: bool,
    gen: G,
    predicate: P,
}

impl<G, P> TakeWhile<G, P> {
    pub(crate) fn new(gen: G, predicate: P) -> Self {
        Self {
            gen,
            predicate,
            complete: false,
        }
    }
}

impl<G: Generator, P> Generator for TakeWhile<G, P>
where
    P: FnMut(&G::Yield) -> bool,
{
    type Yield = G::Yield;
    type Return = ();

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        if self.complete {
            return GeneratorState::Complete(());
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
                _ => GeneratorState::Complete(()),
            }
        }
    }
}
