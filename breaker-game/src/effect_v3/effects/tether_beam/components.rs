//! Tether beam runtime components.

use bevy::prelude::*;

/// Identifies a tether beam entity and the two bolts it connects.
#[derive(Component, Debug, Clone)]
pub struct TetherBeamSource {
    /// First bolt endpoint.
    pub bolt_a: Entity,
    /// Second bolt endpoint.
    pub bolt_b: Entity,
}

/// Damage dealt per tick to cells that cross the tether beam.
#[derive(Component, Debug, Clone)]
pub struct TetherBeamDamage(pub f32);
