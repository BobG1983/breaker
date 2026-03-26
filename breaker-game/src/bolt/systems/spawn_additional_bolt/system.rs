//! System to spawn additional bolt entities from breaker consequences.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{
            Bolt, BoltBaseSpeed, BoltInitialAngle, BoltMaxSpeed, BoltMinSpeed, BoltRadius,
            BoltRespawnAngleSpread, BoltRespawnOffsetY, BoltSpawnOffsetY, ExtraBolt,
            SpawnedByEvolution,
        },
        messages::SpawnAdditionalBolt,
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    run::node::ActiveNodeLayout,
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, EntityScale, GameDrawLayer,
        GameRng, WALL_LAYER,
    },
};

/// Reads [`SpawnAdditionalBolt`] messages and spawns new bolt entities.
///
/// Each bolt spawns above the breaker with a randomized upward velocity
/// at base speed. The bolt is marked [`ExtraBolt`] so it despawns on loss
/// rather than respawning.
pub fn spawn_additional_bolt(
    mut commands: Commands,
    mut reader: MessageReader<SpawnAdditionalBolt>,
    bolt_config: Res<BoltConfig>,
    mut rng: ResMut<GameRng>,
    mut render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    breaker_query: Query<&Position2D, With<Breaker>>,
    layout: Option<Res<ActiveNodeLayout>>,
) {
    let Ok(breaker_pos) = breaker_query.single() else {
        return;
    };
    let breaker_pos = breaker_pos.0;

    let entity_scale = layout.as_ref().map_or(1.0, |l| l.0.entity_scale);

    for msg in reader.read() {
        let angle = rng
            .0
            .random_range(-bolt_config.respawn_angle_spread..=bolt_config.respawn_angle_spread);
        let velocity = Velocity2D(Vec2::new(
            bolt_config.base_speed * angle.sin(),
            bolt_config.base_speed * angle.cos(),
        ));

        let spawn_pos = Vec2::new(breaker_pos.x, breaker_pos.y + bolt_config.spawn_offset_y);

        let id = commands
            .spawn((
                Bolt,
                ExtraBolt,
                velocity,
                GameDrawLayer::Bolt,
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
                EntityScale(entity_scale),
                Mesh2d(render_assets.0.add(Circle::new(1.0))),
                MeshMaterial2d(
                    render_assets
                        .1
                        .add(ColorMaterial::from_color(bolt_config.color())),
                ),
                CleanupOnNodeExit,
            ))
            .id();

        if let Some(name) = &msg.source_chip {
            commands.entity(id).insert(SpawnedByEvolution(name.clone()));
        }
    }
}
