//! Effect v3 system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the effect v3 domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectV3Systems {
    /// Bridge systems that translate game events to trigger dispatches.
    Bridge,
    /// Runtime systems for spawned effect entities (tick, cleanup).
    Tick,
    /// Condition monitoring for During nodes.
    Conditions,
    /// Per-node reset systems (run on state transitions, not in `FixedUpdate` chain).
    Reset,
}
