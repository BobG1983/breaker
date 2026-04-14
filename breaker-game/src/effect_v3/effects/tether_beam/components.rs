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

/// Perpendicular half-width of the tether beam in world units. The beam
/// hit-tests cells whose perpendicular distance from the beam line is
/// less than or equal to this value. Stamped at beam spawn by
/// `TetherBeamConfig::fire_spawn` and `fire_chain`.
#[derive(Component, Debug, Clone)]
pub struct TetherBeamWidth(pub f32);
