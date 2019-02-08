use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

pub struct Take<G> {
    count: usize,
    gen: G,
}

impl<G> Take<G> {
    pub(crate) fn new(gen: G, count: usize) -> Self {
        Self { gen, count }
    }
}

impl<G: Generator> Generator for Take<G> {
    type Yield = G::Yield;
    type Return = ();

    fn resume(self: Pin<&mut Self>) -> GeneratorState<Self::Yield, Self::Return> {
        if self.count != 0 {
            unsafe {
                let this = self.get_unchecked_mut();
                this.count -= 1;
                let gen = Pin::new_unchecked(&mut this.gen);

                match gen.resume() {
                    GeneratorState::Yielded(y) => GeneratorState::Yielded(y),
                    _ => {
                        this.count = 0;
                        GeneratorState::Complete(())
                    }
                }
            }
        } else {
            GeneratorState::Complete(())
        }
    }
}
