#![feature(
    generators,
    generator_trait,
    proc_macro_hygiene,
    stmt_expr_attributes,
    existential_type
)]

pub mod filter;
pub mod iter;
pub mod map;
pub mod mapped;
pub mod take;
pub mod takewhile;

mod generatorext;
pub use generatorext::GeneratorExt;
