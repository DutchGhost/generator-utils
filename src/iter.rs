use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    iter::IntoIterator,
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

        let gen: Pin<&mut Option<G>> = unsafe { _self.map_unchecked_mut(|geniter| &mut geniter.0) };

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

impl <G> Iterator for GenIter<G> where G: Generator + Unpin {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        Pin::new(self).next()
    }
}

#[test]
fn test_iter() {
    let mut g = || {
        for i in 0..5u32 {
            yield i
        }
    };

    let mut iter = GenIter::new(g);

    for (n, i) in (0..5).zip(iter.by_ref()) {
        assert!(n == i);
    }

    // we've gone trough all items of the generator, and we don't panick if we still call .next()
    assert!(iter.next().is_none());
}