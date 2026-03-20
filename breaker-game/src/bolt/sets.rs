//! Bolt domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the bolt domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BoltSystems {
    /// The `init_bolt_params` system — stamps config-derived components on bolt entities.
    InitParams,
    /// The `prepare_bolt_velocity` system — copies bolt velocity for physics.
    PrepareVelocity,
    /// The `reset_bolt` system — resets bolt position and velocity at node start.
    Reset,
}
