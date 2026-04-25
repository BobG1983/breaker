//! Tether hazard — pairs of cells share damage.
//!
//! Design doc: `docs/design/hazards/tether.md`.
//!
//! Owns the `TetherConfig` resource, `TetherLink` component, and the
//! `establish_tether_links` / `cleanup_broken_tether_links` /
//! `tether_emit_partner` systems registered via [`wire`]. The
//! `tether_emit_partner` system runs in `DmgSystems::PostApplyDamage`, reading the
//! current-frame `DamageDealt<Cell>` messages and emitting a partner sibling
//! message that traverses the full damage pipeline on the next `FixedUpdate`
//! tick (1-frame delay).

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
