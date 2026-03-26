//! Shared helper functions for bridge systems.
//!
//! These helpers are used by multiple trigger files under `effect/triggers/`.

mod fns;

#[cfg(test)]
mod tests;

pub(super) use fns::*;
