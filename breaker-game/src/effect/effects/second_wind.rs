//! Invisible bottom wall that bounces bolt once, preventing bolt loss.

use bevy::prelude::*;

/// Marker for the invisible wall entity spawned by Second Wind.
#[derive(Component)]
pub struct SecondWindWall;

/// Spawns an invisible wall at the bottom of the playfield.
///
/// Permanent until used — bounces one bolt, then despawns.
pub(crate) fn fire(_entity: Entity, world: &mut World) {
    let wall = world
        .spawn((
            SecondWindWall,
            Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
        ))
        .id();

    info!("spawned second wind wall {:?}", wall);
}

/// Despawns all `SecondWindWall` entities.
pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let walls: Vec<Entity> = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(world)
        .collect();

    for wall in walls {
        world.despawn(wall);
    }
}

/// Registers systems for `SecondWind` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_spawns_second_wind_wall() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, &mut world);

        let walls: Vec<Entity> = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            walls.len(),
            1,
            "fire should spawn one SecondWindWall entity"
        );
    }

    #[test]
    fn reverse_despawns_all_second_wind_walls() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        world.spawn((
            SecondWindWall,
            Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
        ));
        world.spawn((
            SecondWindWall,
            Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
        ));

        reverse(entity, &mut world);

        let remaining: Vec<Entity> = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert!(
            remaining.is_empty(),
            "reverse should despawn all SecondWindWall entities"
        );
    }
}
