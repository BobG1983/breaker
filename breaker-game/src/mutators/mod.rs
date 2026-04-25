//! Mutators domain — protocols and hazards that mutate the rules of a run.
//!
//! Protocols are positive selectable upgrades chosen at chip-select. Hazards
//! are negative stackable challenges chosen during infinite play at tier 9+.
//! Both share structural patterns and the cross-cutting `MutateDamage` /
//! `PostApplyDamage` chains, which is why they live under one domain owned
//! by [`MutatorsPlugin`].

pub mod hazards;
pub mod plugin;
pub mod protocols;

pub use plugin::MutatorsPlugin;
