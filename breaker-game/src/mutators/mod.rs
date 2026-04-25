//! Mutators domain — protocols and hazards that mutate the rules of a run.
//!
//! Protocols are positive selectable upgrades chosen at chip-select. Hazards
//! are negative stackable challenges chosen during infinite play at tier 9+.
//! Both share structural patterns and the cross-cutting `MutateDamage` /
//! `PostApplyDamage` chains, which is why they live under one domain.
//!
//! Wave 2 will introduce `MutatorsPlugin` here. Until then, the per-subdomain
//! `HazardPlugin` (`hazards::plugin`) and `ProtocolPlugin` (`protocols::plugin`)
//! continue to register independently from `game/system.rs`.

pub mod hazards;
pub mod protocols;
