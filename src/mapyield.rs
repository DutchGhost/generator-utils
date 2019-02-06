use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

#[derive(Clone)]
pub struct MapYield<G, F> {
    g: G,
    f: F,
}

impl<G, F> MapYield<G, F> {
    pub(crate) fn new(g: G, f: F) -> Self {
        Self { g, f }
    }
}

impl<G: Generator, F, O> Generator for MapYield<G, F>
where
    F: FnMut(G::Yield) -> O,
{
    type Yield = O;
    type Return = G::Return;

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        // Unsafe, because we somehow need to (mutably) access the fields of `Self`,
        // while we didn't specify `Self` to be Unpin.
        unsafe {
            let _self: &mut Self = self.get_unchecked_mut();
            let gen: Pin<&mut G> = Pin::new_unchecked(&mut _self.g);

            match gen.resume() {
                GeneratorState::Yielded(y) => GeneratorState::Yielded((_self.f)(y)),
                GeneratorState::Complete(r) => GeneratorState::Complete(r),
            }
        }
    }
}
