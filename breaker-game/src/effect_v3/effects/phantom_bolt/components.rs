//! Phantom bolt runtime components.

use bevy::prelude::*;

/// Marker identifying an entity as a phantom bolt.
///
/// Two spawn paths exist.
/// `effect_v3::effects::phantom_bolt::SpawnPhantomConfig::fire` (chip-effect
/// path) omits `CELL_LAYER` from the collision mask so these phantoms do not
/// interact with cells.
/// `protocol/protocols/afterimage::afterimage_spawn_phantom_bolt` (protocol
/// path) includes `CELL_LAYER` and relies on the `PhantomBolt` branch in
/// `bolt_cell_collision` to pierce-and-damage cells.
/// If a third spawn path appears, this divergence must be reconsidered.
#[derive(Component, Debug, Clone)]
pub struct PhantomBolt;

/// Remaining lifetime in seconds before the phantom bolt despawns.
#[derive(Component, Debug, Clone)]
pub struct PhantomLifetime(pub f32);

/// Entity that spawned this phantom bolt (for ownership tracking).
#[derive(Component, Debug, Clone)]
pub struct PhantomOwner(pub Entity);
