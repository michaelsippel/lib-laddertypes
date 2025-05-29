#![allow(mixed_script_confusables)]
#![allow(confusable_idents)]
#![allow(non_snake_case)]

pub mod term;
pub mod context;
pub mod constraint_system;
pub mod morphism_graph;

pub mod heuristic;

#[cfg(test)]
mod test;

pub use {
    context::*,
    desugared_term::*,
    term::*,
    constraint_system::*,
    morphism_graph::*,
};
