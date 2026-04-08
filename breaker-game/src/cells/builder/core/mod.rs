//! Typestate builder for cell entity construction.
//!
//! Entry point: [`Cell::builder()`]. The builder prevents invalid combinations
//! at compile time via four typestate dimensions: Position, Dimensions, Health,
//! and Visual. `build()` and `spawn()` are only available when all dimensions
//! are satisfied.

pub(crate) mod terminal;
pub(crate) mod transitions;
pub(crate) mod types;

#[cfg(test)]
pub(crate) use types::*;
