//! System to spawn tethered chain bolt entities.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, constraint::DistanceConstraint,
};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{components::*, messages::SpawnChainBolt, resources::BoltConfig},
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameRng, WALL_LAYER},
};

/// Reads [`SpawnChainBolt`] messages and spawns tethered chain bolt entities.
///
/// Each chain bolt spawns at the anchor bolt's position plus a small upward
/// offset, with a randomized upward velocity at base speed. A standalone
/// `DistanceConstraint` entity is created to tether the new bolt to the
/// anchor.
///
/// The chain bolt is marked `ExtraBolt` so it despawns on loss rather than
/// respawning.
pub(crate) fn spawn_chain_bolt(
    mut commands: Commands,
    mut reader: MessageReader<SpawnChainBolt>,
    bolt_config: Res<BoltConfig>,
    mut rng: ResMut<GameRng>,
    bolt_query: Query<&Position2D, With<Bolt>>,
) {
    for msg in reader.read() {
        // Skip if anchor bolt no longer exists
        let Ok(anchor_pos) = bolt_query.get(msg.anchor) else {
            continue;
        };

        let angle = rng
            .0
            .random_range(-bolt_config.respawn_angle_spread..=bolt_config.respawn_angle_spread);
        let velocity = Velocity2D(Vec2::new(
            bolt_config.base_speed * angle.sin(),
            bolt_config.base_speed * angle.cos(),
        ));

        let spawn_pos = Vec2::new(anchor_pos.0.x, anchor_pos.0.y + bolt_config.spawn_offset_y);

        let new_bolt = commands
            .spawn((
                Bolt,
                ExtraBolt,
                velocity,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: bolt_config.radius,
                    y: bolt_config.radius,
                },
                PreviousScale {
                    x: bolt_config.radius,
                    y: bolt_config.radius,
                },
                Aabb2D::new(
                    Vec2::ZERO,
                    Vec2::new(bolt_config.radius, bolt_config.radius),
                ),
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                (
                    BoltBaseSpeed(bolt_config.base_speed),
                    BoltMinSpeed(bolt_config.min_speed),
                    BoltMaxSpeed(bolt_config.max_speed),
                    BoltRadius(bolt_config.radius),
                    BoltSpawnOffsetY(bolt_config.spawn_offset_y),
                    BoltRespawnOffsetY(bolt_config.respawn_offset_y),
                    BoltRespawnAngleSpread(bolt_config.respawn_angle_spread),
                    BoltInitialAngle(bolt_config.initial_angle),
                ),
                CleanupOnNodeExit,
            ))
            .id();

        if let Some(name) = &msg.source_chip {
            commands
                .entity(new_bolt)
                .insert(SpawnedByEvolution(name.clone()));
        }

        // Standalone constraint entity linking anchor to new bolt
        commands.spawn((
            DistanceConstraint {
                entity_a: msg.anchor,
                entity_b: new_bolt,
                max_distance: msg.tether_distance,
            },
            CleanupOnNodeExit,
        ));
    }
}
