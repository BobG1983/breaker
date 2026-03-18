//! System to spawn the bolt entity.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    bolt::{
        components::{Bolt, BoltServing, BoltVelocity},
        resources::BoltConfig,
    },
    breaker::{BreakerConfig, components::Breaker},
    interpolate::components::{InterpolateTransform, PhysicsTranslation},
    run::RunState,
    shared::CleanupOnNodeExit,
};

/// Spawns the bolt entity above the breaker.
///
/// Reads the breaker's Y position from its [`Transform`] when available,
/// falling back to [`BreakerConfig::y_position`] when the breaker entity
/// does not exist yet (both systems run on `OnEnter(Playing)` and deferred
/// commands mean the breaker entity may not exist yet).
///
/// On the first node (`RunState.node_index == 0`), the bolt spawns with
/// zero velocity and a [`BoltServing`] marker — it hovers until the player
/// presses the bump button. On subsequent nodes it launches immediately.
pub fn spawn_bolt(
    mut commands: Commands,
    config: Res<BoltConfig>,
    breaker_config: Res<BreakerConfig>,
    run_state: Res<RunState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    breaker_query: Query<&Transform, With<Breaker>>,
) {
    let breaker_y = breaker_query
        .iter()
        .next()
        .map_or(breaker_config.y_position, |tf| tf.translation.y);

    let spawn_pos = Vec3::new(0.0, breaker_y + config.spawn_offset_y, 1.0);

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
        InterpolateTransform,
        PhysicsTranslation::new(spawn_pos),
        Mesh2d(meshes.add(Circle::new(1.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(config.color()))),
        Transform {
            translation: spawn_pos,
            scale: Vec3::new(config.radius, config.radius, 1.0),
            ..default()
        },
        CleanupOnNodeExit,
    ));
    debug!("bolt spawned entity={:?}", entity.id());

    if serving {
        entity.insert(BoltServing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::Bolt;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
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
    fn bolt_spawns_above_breaker_y() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let breaker_y = BreakerConfig::default().y_position;
        let bolt_transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");
        assert!(bolt_transform.translation.y > breaker_y);
    }

    #[test]
    fn bolt_spawns_at_z_above_zero() {
        let mut app = test_app();
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let bolt_transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");
        assert!(
            bolt_transform.translation.z > 0.0,
            "bolt z should be above 0 to render in front of breaker/cells, got {}",
            bolt_transform.translation.z
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
    fn spawn_does_not_depend_on_breaker_entity() {
        let mut app = test_app();
        // No breaker entity spawned — bolt should still spawn using config
        app.add_systems(Startup, spawn_bolt);
        app.update();

        let count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
        assert_eq!(count, 1, "bolt should spawn even without a breaker entity");
    }
}
