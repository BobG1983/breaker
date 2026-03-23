//! System to spawn the bolt entity.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{
    InterpolateTransform2D, Position2D, PreviousPosition, Scale2D, Spatial2D,
};
use tracing::debug;

use crate::{
    bolt::{
        components::{Bolt, BoltServing, BoltVelocity},
        messages::BoltSpawned,
        resources::BoltConfig,
    },
    breaker::{BreakerConfig, components::Breaker},
    run::RunState,
    shared::{CleanupOnRunEnd, GameDrawLayer},
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
        BoltVelocity::new(0.0, 0.0)
    } else {
        let v = config.initial_velocity();
        BoltVelocity::new(v.x, v.y)
    };

    let mut entity = commands.spawn((
        Bolt,
        velocity,
        Spatial2D,
        InterpolateTransform2D,
        GameDrawLayer::Bolt,
        Position2D(spawn_pos),
        PreviousPosition(spawn_pos),
        Scale2D {
            x: config.radius,
            y: config.radius,
        },
        Mesh2d(render_assets.0.add(Circle::new(1.0))),
        MeshMaterial2d(
            render_assets
                .1
                .add(ColorMaterial::from_color(config.color())),
        ),
        Transform::default(),
        CleanupOnRunEnd,
    ));
    debug!("bolt spawned entity={:?}", entity.id());

    if serving {
        entity.insert(BoltServing);
    }

    bolt_spawned.write(BoltSpawned);
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
    use crate::{bolt::components::Bolt, shared::GameDrawLayer};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltSpawned>()
            .init_resource::<BoltConfig>()
            .init_resource::<BreakerConfig>()
            .init_resource::<RunState>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>();
        app
    }

    #[test]
    fn spawn_bolt_creates_entity() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawned_bolt_has_position2d_at_spawn_position() {
        // Given: no existing bolt, breaker at default y_position (-250.0),
        //        BoltConfig default spawn_offset_y = 30.0
        // When: spawn_bolt runs
        // Then: Bolt has Position2D(Vec2::new(0.0, -220.0))
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let position = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist with Position2D");
        let breaker_y = BreakerConfig::default().y_position; // -250.0
        let spawn_offset_y = BoltConfig::default().spawn_offset_y; // 30.0
        let expected = Vec2::new(0.0, breaker_y + spawn_offset_y); // (0.0, -220.0)
        assert!(
            (position.0.x - expected.x).abs() < f32::EPSILON
                && (position.0.y - expected.y).abs() < f32::EPSILON,
            "bolt Position2D should be {expected:?}, got {:?}",
            position.0,
        );
    }

    #[test]
    fn spawned_bolt_has_position2d_without_breaker_entity() {
        // Edge case: no breaker entity — uses BreakerConfig default y_position
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let position = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist with Position2D even without breaker");
        let expected_y = BreakerConfig::default().y_position + BoltConfig::default().spawn_offset_y;
        assert!(
            (position.0.y - expected_y).abs() < f32::EPSILON,
            "bolt y should use BreakerConfig default, expected {expected_y}, got {}",
            position.0.y,
        );
    }

    #[test]
    fn spawned_bolt_has_game_draw_layer_bolt() {
        // When: spawn_bolt runs
        // Then: Bolt has GameDrawLayer::Bolt with z() == 1.0
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");
        let layer = app
            .world()
            .get::<GameDrawLayer>(entity)
            .expect("bolt should have GameDrawLayer");
        assert!(
            (layer.z() - 1.0).abs() < f32::EPSILON,
            "GameDrawLayer::Bolt.z() should be 1.0, got {}",
            layer.z(),
        );
    }

    #[test]
    fn spawned_bolt_has_spatial2d_and_interpolate_transform2d() {
        // When: spawn_bolt runs
        // Then: Bolt has Spatial2D and InterpolateTransform2D markers, plus
        //       required components Position2D, Rotation2D, Scale2D,
        //       PreviousPosition, PreviousRotation
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");
        let world = app.world();
        assert!(
            world.get::<Spatial2D>(entity).is_some(),
            "bolt should have Spatial2D marker"
        );
        assert!(
            world.get::<InterpolateTransform2D>(entity).is_some(),
            "bolt should have InterpolateTransform2D marker"
        );
        assert!(
            world.get::<Position2D>(entity).is_some(),
            "bolt should have Position2D (via Spatial2D #[require])"
        );
        assert!(
            world.get::<Rotation2D>(entity).is_some(),
            "bolt should have Rotation2D (via Spatial2D #[require])"
        );
        assert!(
            world.get::<Scale2D>(entity).is_some(),
            "bolt should have Scale2D (via Spatial2D #[require])"
        );
        assert!(
            world.get::<PreviousPosition>(entity).is_some(),
            "bolt should have PreviousPosition (via Spatial2D #[require])"
        );
    }

    #[test]
    fn spawned_bolt_previous_position_matches_initial_position() {
        // Edge case: PreviousPosition.0 must match Position2D.0 to prevent
        // interpolation teleport on the first frame
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");
        let pos = app
            .world()
            .get::<Position2D>(entity)
            .expect("bolt should have Position2D");
        let prev = app
            .world()
            .get::<PreviousPosition>(entity)
            .expect("bolt should have PreviousPosition");
        assert_eq!(
            pos.0, prev.0,
            "PreviousPosition should match initial Position2D to prevent teleport"
        );
    }

    #[test]
    fn spawned_bolt_has_scale2d_matching_radius() {
        // Given: BoltConfig default radius = 8.0 (from BoltDefaults)
        // When: spawn_bolt runs
        // Then: Scale2D { x: 8.0, y: 8.0 }
        let mut app = test_app();
        // Use radius = 6.0 from spec for concreteness
        app.world_mut().resource_mut::<BoltConfig>().radius = 6.0;
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let scale = app
            .world_mut()
            .query_filtered::<&Scale2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have Scale2D");
        assert!(
            (scale.x - 6.0).abs() < f32::EPSILON && (scale.y - 6.0).abs() < f32::EPSILON,
            "Scale2D should be (6.0, 6.0), got ({}, {})",
            scale.x,
            scale.y,
        );
    }

    #[test]
    fn bolt_spawns_above_moved_breaker() {
        // Given: breaker at (50.0, -100.0, 0.0), spawn_offset_y = 30.0
        // When: spawn_bolt runs
        // Then: Position2D(Vec2::new(50.0, -70.0))
        let moved_y = -100.0;
        let mut app = test_app();
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(50.0, moved_y)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let config = BoltConfig::default();
        let position = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist with Position2D");
        let expected = Vec2::new(50.0, moved_y + config.spawn_offset_y);
        assert!(
            (position.0.x - expected.x).abs() < f32::EPSILON
                && (position.0.y - expected.y).abs() < f32::EPSILON,
            "bolt Position2D should be {expected:?}, got {:?}",
            position.0,
        );
    }

    #[test]
    fn first_node_spawns_serving_bolt_with_zero_velocity() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let serving_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
            .iter(app.world())
            .count();
        assert_eq!(serving_count, 1, "first node bolt should have BoltServing");

        let velocity = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(
            velocity.speed() < f32::EPSILON,
            "serving bolt should have zero velocity"
        );
    }

    #[test]
    fn subsequent_node_spawns_launched_bolt() {
        let mut app = test_app();
        app.world_mut().resource_mut::<RunState>().node_index = 1;
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let serving_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Bolt>, With<BoltServing>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            serving_count, 0,
            "subsequent node bolt should not have BoltServing"
        );

        let velocity = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");
        assert!(velocity.value.y > 0.0, "bolt should launch upward");
    }

    #[test]
    fn spawn_bolt_sends_bolt_spawned_message() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let messages = app.world().resource::<Messages<BoltSpawned>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "spawn_bolt must send BoltSpawned message"
        );
    }

    #[test]
    fn existing_bolt_is_not_double_spawned() {
        use crate::{
            chips::components::{DamageBoost, Piercing},
            shared::CleanupOnRunEnd,
        };

        let mut app = test_app();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            CleanupOnRunEnd,
            Piercing(2),
            DamageBoost(0.5),
        ));
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
        assert_eq!(
            bolt_count, 1,
            "spawn_bolt should not create a second bolt when one already exists"
        );
    }

    #[test]
    fn existing_bolt_still_triggers_bolt_spawned_message() {
        let mut app = test_app();
        app.world_mut().spawn((Bolt, BoltVelocity::new(0.0, 400.0)));
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let messages = app.world().resource::<Messages<BoltSpawned>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "spawn_bolt must send BoltSpawned even when bolt already exists"
        );
    }

    #[test]
    fn first_spawn_creates_bolt_with_cleanup_on_run_end() {
        use crate::shared::{CleanupOnNodeExit, CleanupOnRunEnd};

        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let entity = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");

        assert!(
            app.world().get::<CleanupOnRunEnd>(entity).is_some(),
            "bolt should have CleanupOnRunEnd marker (persists across nodes)"
        );
        assert!(
            app.world().get::<CleanupOnNodeExit>(entity).is_none(),
            "bolt should NOT have CleanupOnNodeExit (it persists across nodes)"
        );
    }
}
