//! Typestate builder core for wall entity construction.

pub(crate) mod terminal;
pub(crate) mod transitions;
pub(crate) mod types;

#[cfg(test)]
pub(crate) use types::*;
