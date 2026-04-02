//! Typestate builder for breaker entity construction.
//!
//! Entry point: [`Breaker::builder()`]. The builder prevents invalid
//! combinations at compile time via seven typestate dimensions: Dimensions,
//! Movement, Dashing, Spread, Bump, Visual, and Role. `build()` and `spawn()`
//! are only available when all dimensions are satisfied.

pub(crate) mod terminal;
pub(crate) mod transitions;
pub(crate) mod types;

#[cfg(test)]
pub(crate) use types::*;
