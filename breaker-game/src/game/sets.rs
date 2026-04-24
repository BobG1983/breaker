//! Game-level `SystemSet` enums that coordinate cross-domain scheduling.
//!
//! Per `docs/architecture/layout.md`, game-wide sets live in `game/sets.rs`
//! (the same `sets.rs` convention used by individual domains). Consumers
//! that tag systems here must depend on `crate::game::sets::*` (or
//! `crate::game::PostApplyRipple` via the re-export in `game/mod.rs`).

use bevy::prelude::*;

/// Sub-sets of `DmgSystems::PostApplyDamage` that enforce the ordering
/// `diffusion → tether → echo_strike` for ripple emitters. Each mechanic's
/// `register` function tags its emitter with the matching set; the chain
/// edge between the sets is configured by `DmgGameOrderingPlugin` in
/// `game/system.rs`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PostApplyRipple {
    /// `diffusion_emit_rings` runs first inside `PostApplyDamage`.
    Diffusion,
    /// `tether_emit_partner` runs after diffusion.
    Tether,
    /// `echo_strike_emit_siblings` runs last inside `PostApplyDamage`.
    EchoStrike,
}
