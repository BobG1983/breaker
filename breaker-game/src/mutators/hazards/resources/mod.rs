//! Hazard resources — registries, active stacks, offers, and run-condition helpers.

pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use types::{ActiveHazards, HazardRegistry};
pub(crate) use types::{HazardOffers, hazard_active};
