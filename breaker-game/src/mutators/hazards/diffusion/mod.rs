//! Diffusion hazard — multi-frame damage-share redistribution.
//!
//! Per the design doc at `docs/design/hazards/diffusion.md`, redistribution
//! is split across two systems in the `rantzsoft_dmg` pipeline:
//! - `diffusion_reduce_primary` in `DmgSystems::MutateDamage` reduces the
//!   primary `DamageDealt<Cell>.amount` and queues a `PendingEmission`.
//! - `diffusion_emit_rings` in `DmgSystems::PostApplyDamage` drains the queue and
//!   emits one ring sibling per candidate neighbor.
//!
//! `DiffusionInstances` and `PendingDiffusionEmissions` are the two
//! resources that carry state across the two stages. Both reset on
//! `OnExit(NodeState::Playing)` via `reset_diffusion_state`.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
