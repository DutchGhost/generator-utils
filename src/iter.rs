use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

pub struct GenIter<G>(Option<G>);

impl<G: Generator> GenIter<G> {
    pub fn new(gen: G) -> Self {
        Self(Some(gen))
    }
}

impl<'a, G: Generator> Iterator for Pin<&mut GenIter<G>> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {

        let _self: Pin<&mut GenIter<G>> = self.as_mut();

        let mut gen: Pin<&mut Option<G>> = unsafe { _self.map_unchecked_mut(|geniter| &mut geniter.0) };

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
