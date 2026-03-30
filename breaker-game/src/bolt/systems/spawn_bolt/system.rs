//! System to spawn the bolt entity.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};
use tracing::debug;

use crate::{
    bolt::{
        components::{Bolt, BoltServing},
        messages::BoltSpawned,
        resources::BoltConfig,
    },
    breaker::{BreakerConfig, components::Breaker},
    run::RunState,
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnRunEnd, GameDrawLayer, WALL_LAYER},
};

/// Spawns the bolt entity above the breaker.
///
/// Reads the breaker's Y position from its [`Position2D`] when available,
/// falling back to [`BreakerConfig::y_position`] when the breaker entity
/// does not exist yet (both systems run on `OnEnter(Playing)` and deferred
/// commands mean the breaker entity may not exist yet).
///
/// On the first node (`RunState.node_index == 0`), the bolt spawns with
/// zero velocity and a [`BoltServing`] marker — it hovers until the player
/// presses the bump button. On subsequent nodes it launches immediately.
pub(crate) fn spawn_bolt(
    mut commands: Commands,
    configs: (Res<BoltConfig>, Res<BreakerConfig>),
    run_state: Res<RunState>,
    mut render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    breaker_query: Query<&Position2D, With<Breaker>>,
    existing: Query<Entity, With<Bolt>>,
    mut bolt_spawned: MessageWriter<BoltSpawned>,
) {
    let (config, breaker_config) = configs;
    if !existing.is_empty() {
        bolt_spawned.write(BoltSpawned);
        return;
    }

    let breaker_y = breaker_query
        .iter()
        .next()
        .map_or(breaker_config.y_position, |pos| pos.0.y);

    let breaker_x = breaker_query.iter().next().map_or(0.0, |pos| pos.0.x);

    let spawn_pos = Vec2::new(breaker_x, breaker_y + config.spawn_offset_y);

    let serving = run_state.node_index == 0;

    let velocity = if serving {
        Velocity2D(Vec2::new(0.0, 0.0))
    } else {
        let v = config.initial_velocity();
        Velocity2D(Vec2::new(v.x, v.y))
    };

    let mut entity = commands.spawn((
        Bolt,
        velocity,
        GameDrawLayer::Bolt,
        Position2D(spawn_pos),
        PreviousPosition(spawn_pos),
        Scale2D {
            x: config.radius,
            y: config.radius,
        },
        PreviousScale {
            x: config.radius,
            y: config.radius,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::new(config.radius, config.radius)),
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        Mesh2d(render_assets.0.add(Circle::new(1.0))),
        MeshMaterial2d(
            render_assets
                .1
                .add(ColorMaterial::from_color(config.color())),
        ),
        CleanupOnRunEnd,
    ));
    debug!("bolt spawned entity={:?}", entity.id());

    if serving {
        entity.insert(BoltServing);
    }

    bolt_spawned.write(BoltSpawned);
}
