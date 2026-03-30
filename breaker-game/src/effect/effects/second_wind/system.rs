//! Invisible bottom wall that bounces bolt once, preventing bolt loss.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use crate::{
    bolt::messages::BoltImpactWall,
    shared::{BOLT_LAYER, PlayfieldConfig, WALL_LAYER},
    wall::components::{Wall, WallSize},
};

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

    let playfield = world.resource::<PlayfieldConfig>();
    let bottom_y = playfield.bottom();
    let half_width = playfield.width / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    let wall = world
        .spawn((
            SecondWindWall,
            Wall,
            WallSize {},
            Position2D(Vec2::new(0.0, bottom_y)),
            Scale2D {
                x: half_width,
                y: wall_ht,
            },
            Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, wall_ht)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        ))
        .id();

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
    use crate::{bolt::BoltSystems, shared::playing_state::PlayingState};
    app.add_systems(
        FixedUpdate,
        despawn_second_wind_on_contact
            .run_if(in_state(PlayingState::Active))
            .after(BoltSystems::WallCollision),
    );
}
