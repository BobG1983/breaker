//! Unified size computation — pure functions for entity dimensions.
//!
//! `effective_size` is the single source of truth
//! for computing entity dimensions from base values, boost multipliers,
//! node scaling, and optional min/max clamping.

pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use types::{
    BaseRadius, ClampRange, MaxHeight, MaxRadius, MaxWidth, MinHeight, MinRadius, MinWidth,
};
pub(crate) use types::{effective_radius, effective_size};
