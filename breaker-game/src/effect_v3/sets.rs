//! Effect v3 system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the effect v3 domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectV3Systems {
    /// Bridge systems that translate game events to trigger dispatches
    /// (bump, impact, bolt-lost, time, node). Scheduled at the top of
    /// `FixedUpdate` before `Tick`. Death bridges are NOT in this set —
    /// see [`EffectV3Systems::Death`] for why.
    Bridge,
    /// Runtime systems for spawned effect entities (tick, cleanup).
    Tick,
    /// Condition monitoring for During nodes.
    Conditions,
    /// Death-trigger bridges (`on_cell_destroyed`, `on_bolt_destroyed`,
    /// `on_wall_destroyed`, `on_breaker_destroyed`). Scheduled
    /// `.after(DeathPipelineSystems::HandleKill)` so the bridges observe
    /// `Destroyed<T>` messages on the same tick they are written, while
    /// the victim entity still exists in the world (despawn happens later
    /// in `FixedPostUpdate`). This is the tagging surface cross-domain
    /// consumers use to order against the death-bridge phase.
    Death,
    /// Per-node reset systems (run on state transitions, not in `FixedUpdate` chain).
    Reset,
}
