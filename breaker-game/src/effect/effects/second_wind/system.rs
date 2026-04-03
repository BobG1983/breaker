//! Invisible bottom wall that bounces bolt once, preventing bolt loss.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{bolt::messages::BoltImpactWall, shared::PlayfieldConfig, walls::components::Wall};

/// Marker for the invisible wall entity spawned by Second Wind.
#[derive(Component)]
pub struct SecondWindWall;

/// Spawns an invisible wall at the bottom of the playfield.
///
/// Permanent until used — bounces one bolt, then despawns.
/// If a `SecondWindWall` entity already exists, skips spawning.
pub(crate) fn fire(_entity: Entity, _source_chip: &str, world: &mut World) {
    // Guard: do not spawn a second wall if one already exists.
    let existing = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(world)
        .count();
    if existing > 0 {
        return;
    }

    let playfield = world.resource::<PlayfieldConfig>().clone();
    let bundle = Wall::builder()
        .floor(&playfield)
        .invisible()
        .one_shot()
        .build();
    let wall = world.spawn((SecondWindWall, bundle)).id();

    info!("spawned second wind wall {:?}", wall);
}

/// Despawns all `SecondWindWall` entities.
pub(crate) fn reverse(_entity: Entity, _source_chip: &str, world: &mut World) {
    let walls: Vec<Entity> = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(world)
        .collect();

    for wall in walls {
        world.despawn(wall);
    }
}

/// Despawns a [`SecondWindWall`] entity after a bolt collides with it.
///
/// Reads [`BoltImpactWall`] messages and checks if the impacted wall
/// has the [`SecondWindWall`] marker. If so, despawns it — making the
/// `SecondWind` effect single-use.
///
/// Uses a `Local<HashSet<Entity>>` to track walls already scheduled for despawn
/// this frame, preventing double-despawn when two bolts hit the same wall.
pub(crate) fn despawn_second_wind_on_contact(
    mut commands: Commands,
    mut reader: MessageReader<BoltImpactWall>,
    wall_query: Query<Entity, With<SecondWindWall>>,
    mut despawned: Local<HashSet<Entity>>,
) {
    // Local<HashSet> reuses its heap allocation across frames — zero allocs after warmup.
    despawned.clear();
    for msg in reader.read() {
        if despawned.contains(&msg.wall) {
            continue;
        }
        if wall_query.get(msg.wall).is_ok() {
            despawned.insert(msg.wall);
            commands.entity(msg.wall).despawn();
        }
    }
}

/// Registers systems for `SecondWind` effect.
pub(crate) fn register(app: &mut App) {
    use crate::{bolt::BoltSystems, state::types::NodeState};
    app.add_systems(
        FixedUpdate,
        despawn_second_wind_on_contact
            .run_if(in_state(NodeState::Playing))
            .after(BoltSystems::WallCollision),
    );
}
