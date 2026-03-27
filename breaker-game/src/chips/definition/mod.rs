//! Chip definition types — content types for chip definitions and templates.

mod types;

#[cfg(test)]
mod tests;

pub use types::*;
pub(crate) use types::{expand_chip_template, expand_evolution_template};
