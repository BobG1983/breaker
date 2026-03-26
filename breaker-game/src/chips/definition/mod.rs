//! Chip definition types — content types for chip definitions and templates.

mod types;

#[cfg(test)]
mod tests;

pub(crate) use types::expand_template;
pub use types::*;
