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

impl<G: Generator> Iterator for GenIter<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        let pinned: Option<Pin<&mut G>> = self
            .0
            .as_mut()
            .map(|gen| unsafe { Pin::new_unchecked(gen) });

        match pinned.map(Generator::resume) {
            Some(GeneratorState::Yielded(y)) => Some(y),
            Some(GeneratorState::Complete(_)) => {
                self.0 = None;
                None
            }
            None => None,
        }
    }
}
