#![allow(mixed_script_confusables)]

pub mod bimap;
pub mod dict;

pub mod lexer;
pub mod parser; // todo sugared variant
pub mod curry; // todo: sugared variant
//pub mod subtype; // deprecated
pub mod unparser;
pub mod desugared_term; // deprecated
pub mod term;
pub mod pnf;
pub mod substitution;
pub mod constraint_system;
pub mod morphism;
pub mod morphism_base;
pub mod morphism_path;

#[cfg(test)]
mod test;

#[cfg(feature = "pretty")]
mod pretty;

pub use {
    dict::*,
    desugared_term::*,
    substitution::*,
    term::*,
    constraint_system::*,
    morphism::*,
    morphism_base::*,
    morphism_path::*,
};
