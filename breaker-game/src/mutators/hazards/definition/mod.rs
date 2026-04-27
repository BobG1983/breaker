//! Hazard definition — `HazardKind`, `HazardTuning`, `HazardDefinition`.

pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use types::HazardKind;
pub(crate) use types::{HazardDefinition, HazardTuning};
