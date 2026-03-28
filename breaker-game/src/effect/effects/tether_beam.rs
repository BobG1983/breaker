//! Two free-moving bolts connected by a crackling neon beam that damages intersected cells.

use bevy::prelude::*;

/// Spawns two tethered bolts with a damaging beam between them.
///
/// Evolution of `ChainBolt`. The beam is a line segment between the two bolt
/// positions — cells intersecting the beam take damage each tick.
pub(crate) fn fire(entity: Entity, damage_mult: f32, world: &mut World) {
    let _ = (entity, damage_mult, world);
    // Placeholder — real implementation in Wave 8
}

/// No-op — tether bolts have their own lifecycle.
pub(crate) fn reverse(_entity: Entity, _damage_mult: f32, _world: &mut World) {}

/// Registers systems for `TetherBeam` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_completes_without_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        fire(entity, 1.5, &mut world);
    }

    #[test]
    fn reverse_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        reverse(entity, 1.5, &mut world);
    }
}
