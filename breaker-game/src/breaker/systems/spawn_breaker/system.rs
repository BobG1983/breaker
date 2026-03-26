//! System to spawn the breaker entity.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition, PreviousScale, Scale2D};
use tracing::debug;

use crate::{
    breaker::{
        components::{
            Breaker, BreakerState, BreakerStateTimer, BreakerTilt, BreakerVelocity, BumpState,
        },
        messages::BreakerSpawned,
        queries::ResetQuery,
        resources::BreakerConfig,
    },
    shared::{BOLT_LAYER, BREAKER_LAYER, CleanupOnRunEnd, GameDrawLayer, PlayfieldConfig},
};

/// Spawns the breaker entity with all required components.
///
/// Runs when entering [`GameState::Playing`]. If a breaker already exists
/// (persisted from a previous node), this is a no-op.
pub fn spawn_breaker(
    mut commands: Commands,
    config: Res<BreakerConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing: Query<Entity, With<Breaker>>,
    mut breaker_spawned: MessageWriter<BreakerSpawned>,
) {
    if !existing.is_empty() {
        breaker_spawned.write(BreakerSpawned);
        return;
    }

    let entity = commands.spawn((
        // Core breaker components
        (
            Breaker,
            BreakerVelocity::default(),
            BreakerState::default(),
            BreakerTilt::default(),
            BumpState::default(),
            BreakerStateTimer::default(),
        ),
        // Spatial2d components
        (
            GameDrawLayer::Breaker,
            Position2D(Vec2::new(0.0, config.y_position)),
            PreviousPosition(Vec2::new(0.0, config.y_position)),
            Scale2D {
                x: config.width,
                y: config.height,
            },
            PreviousScale {
                x: config.width,
                y: config.height,
            },
        ),
        // Physics
        (
            Aabb2D::new(
                Vec2::ZERO,
                Vec2::new(config.width / 2.0, config.height / 2.0),
            ),
            CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER),
        ),
        // Rendering + cleanup
        (
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(config.color()))),
            CleanupOnRunEnd,
        ),
    ));
    debug!("breaker spawned entity={:?}", entity.id());
    breaker_spawned.write(BreakerSpawned);
}

/// Resets breaker state at the start of each node.
///
/// Runs when entering [`GameState::Playing`]. Returns breaker to center,
/// clears velocity/tilt/state. On the first node, `spawn_breaker` handles
/// initialization — this system is a no-op if no breaker exists yet.
pub fn reset_breaker(playfield: Res<PlayfieldConfig>, mut query: Query<ResetQuery, With<Breaker>>) {
    // Robust if PlayfieldConfig is ever offset from world origin
    let center_x = f32::midpoint(playfield.left(), playfield.right());
    for (mut position, mut state, mut velocity, mut tilt, mut timer, mut bump, base_y, prev) in
        &mut query
    {
        position.0.x = center_x;
        position.0.y = base_y.0;
        *state = BreakerState::Idle;
        velocity.x = 0.0;
        tilt.angle = 0.0;
        tilt.ease_start = 0.0;
        tilt.ease_target = 0.0;
        timer.remaining = 0.0;
        bump.active = false;
        bump.timer = 0.0;
        bump.post_hit_timer = 0.0;
        bump.cooldown = 0.0;
        // Snap interpolation to avoid lerping through teleport
        if let Some(mut prev) = prev {
            *prev = PreviousPosition(position.0);
        }
    }
}
