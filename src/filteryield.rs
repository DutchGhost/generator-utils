use std::{
    pin::Pin,
    ops::{Generator, GeneratorState},
};

pub struct FilterYield<G, F> {
    gen: G,
    predicate: F,
}

impl <G, F> FilterYield<G, F> {
    pub(crate) fn new(gen: G, predicate: F) -> Self {
        Self { gen, predicate }
    }
}