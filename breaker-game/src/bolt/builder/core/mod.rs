//! Typestate builder for bolt entity construction.
//!
//! Entry point: [`Bolt::builder()`]. The builder prevents invalid combinations
//! at compile time via six typestate dimensions: Position, Speed, Angle,
//! Motion, Role, and Visual. `build()` and `spawn()` are only available when
//! all dimensions are satisfied.

pub(crate) mod terminal;
pub(crate) mod transitions;
pub(crate) mod types;

#[cfg(test)]
pub(crate) use types::*;
