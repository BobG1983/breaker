//! Hazard sub-domain — negative stackable hazards chosen during infinite
//! play at tier 9 and above.
//!
//! Each per-hazard module owns one hazard's config resource, activation
//! logic, and runtime system(s). Most are scaffold-level today — the
//! headline runtime behaviour lands as cross-domain plumbing (damage
//! pipeline, cell HP churn, breaker-shrink) stabilises.

pub mod definition;
pub(crate) mod messages;
pub mod resources;
pub(crate) mod systems;

pub(crate) mod fan_out;

pub(crate) mod cascade;
pub(crate) mod decay;
pub(crate) mod diffusion;
pub(crate) mod drift;
pub(crate) mod echo_cells;
pub(crate) mod erosion;
pub(crate) mod fracture;
pub(crate) mod gravity_surge;
pub(crate) mod haste;
pub(crate) mod momentum;
pub(crate) mod overcharge;
pub(crate) mod renewal;
pub(crate) mod resonance;
pub(crate) mod sympathy;
pub(crate) mod tether;
pub(crate) mod volatility;

#[cfg(test)]
mod tests;

pub use fan_out::activate_from_registry;
pub(crate) use fan_out::{activate, wire};
