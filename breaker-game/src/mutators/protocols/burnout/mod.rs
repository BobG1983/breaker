//! Burnout protocol — heat-gauge rhythm mechanic.
//!
//! Design doc: `docs/design/protocols/burnout.md`.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::{BurnoutDamageBoost, BurnoutHeat};
pub(crate) use system::{activate, wire};
