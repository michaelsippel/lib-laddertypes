
pub mod lexer;
pub mod bimap;
pub mod dict;
pub mod term;

#[cfg(test)]
mod test;

pub use {
    dict::*,
    term::*,
};

