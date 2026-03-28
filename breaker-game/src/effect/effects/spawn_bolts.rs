use bevy::prelude::*;

/// Spawns additional bolts from an entity.
///
/// For now this is a placeholder — spawning bolts requires bolt domain
/// infrastructure (`SpawnAdditionalBolt` message).
pub(crate) fn fire(
    entity: Entity,
    count: u32,
    lifespan: Option<f32>,
    inherit: bool,
    _world: &mut World,
) {
    info!(
        "spawn {} bolts from entity {:?} (lifespan: {:?}, inherit: {})",
        count, entity, lifespan, inherit
    );
}

/// No-op — bolts persist independently once spawned.
pub(crate) fn reverse(
    _entity: Entity,
    _count: u32,
    _lifespan: Option<f32>,
    _inherit: bool,
    _world: &mut World,
) {
}

/// Registers systems for `SpawnBolts` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_completes_without_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        // Placeholder effect — should not panic.
        fire(entity, 3, Some(5.0), true, &mut world);
    }

    #[test]
    fn reverse_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        // reverse should complete without panic or side effects.
        reverse(entity, 3, Some(5.0), true, &mut world);
    }
}
