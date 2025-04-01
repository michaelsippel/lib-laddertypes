#![allow(mixed_script_confusables)]

pub mod bimap;
pub mod dict;

pub mod lexer;
pub mod parser;
pub mod unparser;
pub mod curry;

pub mod lnf; // deprecated
pub mod subtype; // deprecated

pub mod pnf;
pub mod pnf_sugared;

pub mod term;
pub mod sugar;

pub mod substitution;
pub mod substitution_sugared;

pub mod unification;
pub mod unification_sugared;

pub mod morphism;
pub mod morphism_sugared;

pub mod morphism_base;
pub mod morphism_base_sugared;

pub mod morphism_path;
pub mod morphism_path_sugared;

#[cfg(test)]
mod test;

#[cfg(feature = "pretty")]
mod pretty;

pub use {
    dict::*,
    term::*,
    substitution::*,
    sugar::*,
    unification::*,
    morphism::*
};
