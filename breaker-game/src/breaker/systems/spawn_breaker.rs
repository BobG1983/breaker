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

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::{
        components::{
            InterpolateTransform2D, Position2D, PreviousPosition, Rotation2D, Scale2D, Spatial2D,
        },
        draw_layer::DrawLayer,
    };

    use super::*;
    use crate::{
        breaker::components::{Breaker, BreakerBaseY},
        shared::GameDrawLayer,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BreakerSpawned>()
            .init_resource::<BreakerConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Startup, spawn_breaker);
        app
    }

    #[test]
    fn spawn_breaker_creates_entity() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query::<&Breaker>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawned_breaker_has_spatial2d_components() {
        // Given: no breaker exists
        // When: spawn_breaker runs
        // Then: breaker has Spatial2D, InterpolateTransform2D, GameDrawLayer::Breaker,
        //       Position2D, PreviousPosition, Scale2D, Rotation2D, Transform::default()
        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        let world = app.world();
        assert!(
            world.get::<Spatial2D>(entity).is_some(),
            "breaker should have Spatial2D marker"
        );
        assert!(
            world.get::<InterpolateTransform2D>(entity).is_some(),
            "breaker should have InterpolateTransform2D marker"
        );
        assert!(
            world.get::<Position2D>(entity).is_some(),
            "breaker should have Position2D"
        );
        assert!(
            world.get::<PreviousPosition>(entity).is_some(),
            "breaker should have PreviousPosition"
        );
        assert!(
            world.get::<Rotation2D>(entity).is_some(),
            "breaker should have Rotation2D"
        );
        assert!(
            world.get::<Scale2D>(entity).is_some(),
            "breaker should have Scale2D"
        );
        let layer = world
            .get::<GameDrawLayer>(entity)
            .expect("breaker should have GameDrawLayer");
        assert!(
            layer.z().abs() < f32::EPSILON,
            "GameDrawLayer::Breaker.z() should be 0.0, got {}",
            layer.z(),
        );
    }

    #[test]
    fn spawned_breaker_has_position2d_at_spawn_position() {
        // Given: BreakerConfig default y_position=-250.0
        // When: spawn_breaker runs
        // Then: Position2D(Vec2::new(0.0, -250.0))
        let mut app = test_app();
        app.update();

        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        let position = app
            .world()
            .get::<Position2D>(entity)
            .expect("breaker should have Position2D");
        let expected = Vec2::new(0.0, config.y_position);
        assert!(
            (position.0.x - expected.x).abs() < f32::EPSILON
                && (position.0.y - expected.y).abs() < f32::EPSILON,
            "breaker Position2D should be {expected:?}, got {:?}",
            position.0,
        );
    }

    #[test]
    fn spawned_breaker_previous_position_matches_initial_position() {
        // Edge case: PreviousPosition.0 must match Position2D.0 to prevent
        // interpolation teleport on the first frame
        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        let pos = app
            .world()
            .get::<Position2D>(entity)
            .expect("breaker should have Position2D");
        let prev = app
            .world()
            .get::<PreviousPosition>(entity)
            .expect("breaker should have PreviousPosition");
        assert_eq!(
            pos.0, prev.0,
            "PreviousPosition should match initial Position2D to prevent teleport"
        );
    }

    #[test]
    fn spawned_breaker_has_scale2d_matching_dimensions() {
        // Given: BreakerConfig default width=120.0, height=20.0
        // When: spawn_breaker runs
        // Then: Scale2D { x: 120.0, y: 20.0 }
        let mut app = test_app();
        app.update();

        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        let scale = app
            .world()
            .get::<Scale2D>(entity)
            .expect("breaker should have Scale2D");
        assert!(
            (scale.x - config.width).abs() < f32::EPSILON
                && (scale.y - config.height).abs() < f32::EPSILON,
            "Scale2D should be ({}, {}), got ({}, {})",
            config.width,
            config.height,
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn spawned_breaker_has_default_transform() {
        // After migration, Transform should be default (propagation handles it)
        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        let transform = app
            .world()
            .get::<Transform>(entity)
            .expect("breaker should have Transform");
        assert_eq!(
            *transform,
            Transform::default(),
            "breaker Transform should be default after spatial2d migration, got {transform:?}"
        );
    }

    #[test]
    fn spawn_breaker_sends_breaker_spawned_message() {
        let mut app = test_app();
        app.update();

        let messages = app.world().resource::<Messages<BreakerSpawned>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "spawn_breaker must send BreakerSpawned message"
        );
    }

    #[test]
    fn no_double_spawn() {
        let mut app = test_app();
        app.update();

        // Run spawn_breaker again (simulating a second node entry)
        app.add_systems(Update, spawn_breaker);
        app.update();

        let count = app
            .world_mut()
            .query::<&Breaker>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should not double-spawn breaker");
    }

    #[test]
    fn existing_breaker_still_sends_breaker_spawned() {
        let mut app = test_app();
        app.update(); // First spawn

        // Run spawn_breaker again — breaker already exists
        app.add_systems(Update, spawn_breaker);
        app.update();

        let messages = app.world().resource::<Messages<BreakerSpawned>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "spawn_breaker must send BreakerSpawned even when breaker already exists"
        );
    }

    #[test]
    fn reset_breaker_writes_position2d() {
        // Given: Breaker at Position2D(Vec2::new(100.0, -200.0)), BreakerBaseY(-250.0)
        // When: reset_breaker runs
        // Then: Position2D(Vec2::new(0.0, -250.0))
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .init_resource::<PlayfieldConfig>();

        let config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            PreviousPosition(Vec2::new(100.0, -200.0)),
            BreakerVelocity { x: 300.0 },
            BreakerState::Dashing,
            BreakerTilt {
                angle: 0.5,
                ease_start: 0.5,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.1 },
            BreakerBaseY(config.y_position),
            BumpState {
                active: true,
                timer: 0.1,
                post_hit_timer: 0.05,
                cooldown: 0.2,
                last_hit_bolt: None,
            },
            CleanupOnRunEnd,
        ));

        app.add_systems(Update, reset_breaker);
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        let position = app
            .world()
            .get::<Position2D>(entity)
            .expect("breaker should have Position2D");
        assert!(
            position.0.x.abs() < f32::EPSILON,
            "Position2D.x should be 0.0 after reset, got {}",
            position.0.x,
        );
        assert!(
            (position.0.y - config.y_position).abs() < f32::EPSILON,
            "Position2D.y should be {}, got {}",
            config.y_position,
            position.0.y,
        );
    }

    #[test]
    fn reset_breaker_previous_position_matches_position() {
        // Given: Breaker at Position2D(Vec2::new(100.0, -200.0)) with stale PreviousPosition
        // When: reset_breaker runs
        // Then: PreviousPosition matches Position2D (no interpolation teleport)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .init_resource::<PlayfieldConfig>();

        let config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -200.0)),
            PreviousPosition(Vec2::new(50.0, -180.0)),
            BreakerVelocity { x: 300.0 },
            BreakerState::Dashing,
            BreakerTilt {
                angle: 0.5,
                ease_start: 0.5,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.1 },
            BreakerBaseY(config.y_position),
            BumpState {
                active: true,
                timer: 0.1,
                post_hit_timer: 0.05,
                cooldown: 0.2,
                last_hit_bolt: None,
            },
            CleanupOnRunEnd,
        ));

        app.add_systems(Update, reset_breaker);
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        let position = app
            .world()
            .get::<Position2D>(entity)
            .expect("breaker should have Position2D");
        let prev = app
            .world()
            .get::<PreviousPosition>(entity)
            .expect("breaker should have PreviousPosition");
        assert_eq!(
            position.0, prev.0,
            "PreviousPosition should match Position2D after reset to prevent teleport"
        );
    }

    #[test]
    fn reset_breaker_restores_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BreakerConfig>()
            .init_resource::<PlayfieldConfig>();

        let config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, config.y_position + 50.0)),
            BreakerVelocity { x: 300.0 },
            BreakerState::Dashing,
            BreakerTilt {
                angle: 0.5,
                ease_start: 0.5,
                ease_target: 0.0,
            },
            BreakerStateTimer { remaining: 0.1 },
            BreakerBaseY(config.y_position),
            BumpState {
                active: true,
                timer: 0.1,
                post_hit_timer: 0.05,
                cooldown: 0.2,
                last_hit_bolt: None,
            },
            CleanupOnRunEnd,
        ));

        app.add_systems(Update, reset_breaker);
        app.update();

        let (state, velocity, tilt, timer, bump) = app
            .world_mut()
            .query::<(
                &BreakerState,
                &BreakerVelocity,
                &BreakerTilt,
                &BreakerStateTimer,
                &BumpState,
            )>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");

        assert_eq!(*state, BreakerState::Idle);
        assert!(velocity.x.abs() < f32::EPSILON);
        assert!(tilt.angle.abs() < f32::EPSILON);
        assert!(tilt.ease_start.abs() < f32::EPSILON);
        assert!(timer.remaining.abs() < f32::EPSILON);
        assert!(!bump.active, "bump should be inactive after reset");
        assert!(
            bump.timer.abs() < f32::EPSILON,
            "bump timer should be cleared"
        );
        assert!(
            bump.post_hit_timer.abs() < f32::EPSILON,
            "post_hit_timer should be cleared"
        );
        assert!(
            bump.cooldown.abs() < f32::EPSILON,
            "cooldown should be cleared"
        );
    }

    // --- Aabb2D + CollisionLayers tests ---

    #[test]
    fn spawned_breaker_has_aabb2d_matching_breaker_dimensions() {
        // Given: BreakerConfig default (width=120.0, height=20.0)
        // When: spawn_breaker runs
        // Then: breaker entity has Aabb2D { center: Vec2::ZERO, half_extents: Vec2::new(60.0, 10.0) }
        use rantzsoft_physics2d::aabb::Aabb2D;

        let mut app = test_app();
        app.update();

        let config = BreakerConfig::default();
        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        let aabb = app
            .world()
            .get::<Aabb2D>(entity)
            .expect("breaker should have Aabb2D");
        assert_eq!(
            aabb.center,
            Vec2::ZERO,
            "breaker Aabb2D center should be ZERO (local space)"
        );
        let expected_half_w = config.width / 2.0; // 60.0
        let expected_half_h = config.height / 2.0; // 10.0
        assert!(
            (aabb.half_extents.x - expected_half_w).abs() < f32::EPSILON
                && (aabb.half_extents.y - expected_half_h).abs() < f32::EPSILON,
            "breaker Aabb2D half_extents should be ({expected_half_w}, {expected_half_h}), got ({}, {})",
            aabb.half_extents.x,
            aabb.half_extents.y,
        );
    }

    #[test]
    fn spawned_breaker_has_collision_layers_breaker_membership_bolt_mask() {
        // Given: spawn_breaker runs
        // Then: CollisionLayers { membership: BREAKER_LAYER (0x08), mask: BOLT_LAYER (0x01) }
        use rantzsoft_physics2d::collision_layers::CollisionLayers;

        use crate::shared::{BOLT_LAYER, BREAKER_LAYER};

        let mut app = test_app();
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Breaker>>()
            .iter(app.world())
            .next()
            .expect("breaker should exist");
        let layers = app
            .world()
            .get::<CollisionLayers>(entity)
            .expect("breaker should have CollisionLayers");
        assert_eq!(
            layers.membership, BREAKER_LAYER,
            "breaker membership should be BREAKER_LAYER (0x{:02X}), got 0x{:02X}",
            BREAKER_LAYER, layers.membership,
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "breaker mask should be BOLT_LAYER (0x{:02X}), got 0x{:02X}",
            BOLT_LAYER, layers.mask,
        );
    }
}
