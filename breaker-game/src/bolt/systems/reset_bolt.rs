//! System to reset the bolt's position and velocity at the start of each node.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{Bolt, BoltServing, ExtraBolt},
        queries::ResetBoltQuery,
        resources::BoltConfig,
    },
    breaker::components::Breaker,
    interpolate::components::PhysicsTranslation,
    run::RunState,
};

/// Resets the bolt's position above the breaker and adjusts velocity for the
/// current node.
///
/// On the first node (`RunState.node_index == 0`), the bolt spawns with zero
/// velocity and a [`BoltServing`] marker. On subsequent nodes it launches
/// immediately at base speed.
///
/// Chip effect components (e.g. [`Piercing`], [`DamageBoost`]) are NOT touched
/// — they persist across nodes. Only positional and velocity state is reset.
pub fn reset_bolt(
    mut commands: Commands,
    config: Res<BoltConfig>,
    run_state: Res<RunState>,
    breaker_query: Query<&Transform, (With<Breaker>, Without<Bolt>)>,
    mut bolt_query: Query<ResetBoltQuery, (With<Bolt>, Without<ExtraBolt>)>,
) {
    let (breaker_x, breaker_y) = breaker_query
        .iter()
        .next()
        .map_or((0.0, 0.0), |tf| (tf.translation.x, tf.translation.y));

    for (entity, mut transform, mut velocity, piercing_remaining, piercing, physics_translation) in
        &mut bolt_query
    {
        let new_pos = Vec3::new(
            breaker_x,
            breaker_y + config.spawn_offset_y,
            transform.translation.z,
        );
        transform.translation = new_pos;

        if run_state.node_index == 0 {
            velocity.value = Vec2::ZERO;
            commands.entity(entity).insert(BoltServing);
        } else {
            velocity.value = config.initial_velocity();
            commands.entity(entity).remove::<BoltServing>();
        }

        if let (Some(mut remaining), Some(pierce)) = (piercing_remaining, piercing) {
            remaining.0 = pierce.0;
        }

        if let Some(mut pt) = physics_translation {
            *pt = PhysicsTranslation::new(new_pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::{
        bolt::{
            components::{Bolt, BoltServing, BoltVelocity, ExtraBolt},
            resources::BoltConfig,
        },
        breaker::components::Breaker,
        chips::components::{
            BoltSizeBoost, BoltSpeedBoost, ChainHit, DamageBoost, Piercing, PiercingRemaining,
        },
        interpolate::components::PhysicsTranslation,
        run::RunState,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BoltConfig>()
            .init_resource::<RunState>()
            .add_systems(Update, reset_bolt);
        app
    }

    /// Spawns a bolt entity with standard components for reset testing.
    fn spawn_bolt_entity(app: &mut App, pos: Vec3, velocity: BoltVelocity) -> Entity {
        app.world_mut()
            .spawn((
                Bolt,
                velocity,
                Transform::from_translation(pos),
                PhysicsTranslation::new(pos),
            ))
            .id()
    }

    /// Spawns a breaker entity at the given position.
    fn spawn_breaker(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(x, y, 0.0)))
            .id()
    }

    #[test]
    fn reset_bolt_repositions_bolt_above_breaker() {
        let mut app = test_app();
        let bolt_pos = Vec3::new(150.0, 100.0, 1.0);
        spawn_bolt_entity(&mut app, bolt_pos, BoltVelocity::new(300.0, 400.0));
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let config = BoltConfig::default();
        let expected_y = -250.0 + config.spawn_offset_y; // -250.0 + 30.0 = -220.0

        let transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");

        assert!(
            (transform.translation.x - 0.0).abs() < f32::EPSILON,
            "bolt x should be centered above breaker, got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - expected_y).abs() < f32::EPSILON,
            "bolt y should be {expected_y}, got {}",
            transform.translation.y
        );
        assert!(
            (transform.translation.z - 1.0).abs() < f32::EPSILON,
            "bolt z should remain 1.0, got {}",
            transform.translation.z
        );
    }

    #[test]
    fn reset_bolt_zeroes_velocity_on_node_zero() {
        let mut app = test_app();
        // node_index defaults to 0
        spawn_bolt_entity(
            &mut app,
            Vec3::new(0.0, 0.0, 1.0),
            BoltVelocity::new(300.0, 400.0),
        );
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let velocity = app
            .world_mut()
            .query_filtered::<&BoltVelocity, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");

        assert!(
            velocity.value == Vec2::ZERO,
            "velocity should be zero on node 0, got {:?}",
            velocity.value
        );
    }

    #[test]
    fn reset_bolt_sets_initial_velocity_on_subsequent_nodes() {
        let mut app = test_app();
        app.world_mut().resource_mut::<RunState>().node_index = 2;
        spawn_bolt_entity(
            &mut app,
            Vec3::new(0.0, 0.0, 1.0),
            BoltVelocity::new(0.0, 0.0),
        );
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let config = BoltConfig::default();
        let velocity = app
            .world_mut()
            .query_filtered::<&BoltVelocity, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have velocity");

        assert!(
            velocity.value.y > 0.0,
            "velocity y should be positive on subsequent node, got {}",
            velocity.value.y
        );
        let speed = velocity.speed();
        assert!(
            (speed - config.base_speed).abs() < 1.0,
            "speed should be approximately base_speed ({:.1}), got {speed:.1}",
            config.base_speed
        );
    }

    #[test]
    fn reset_bolt_inserts_serving_on_node_zero() {
        let mut app = test_app();
        // node_index defaults to 0
        let bolt_id = spawn_bolt_entity(
            &mut app,
            Vec3::new(0.0, 0.0, 1.0),
            BoltVelocity::new(0.0, 0.0),
        );
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        assert!(
            app.world().get::<BoltServing>(bolt_id).is_some(),
            "bolt should have BoltServing on node 0"
        );
    }

    #[test]
    fn reset_bolt_removes_serving_on_subsequent_nodes() {
        let mut app = test_app();
        app.world_mut().resource_mut::<RunState>().node_index = 1;
        let bolt_id = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                BoltVelocity::new(0.0, 0.0),
                Transform::from_xyz(0.0, 0.0, 1.0),
                PhysicsTranslation::new(Vec3::new(0.0, 0.0, 1.0)),
            ))
            .id();
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        assert!(
            app.world().get::<BoltServing>(bolt_id).is_none(),
            "bolt should NOT have BoltServing on node 1"
        );
    }

    #[test]
    fn reset_bolt_resets_piercing_remaining_to_piercing() {
        let mut app = test_app();
        let bolt_id = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 0.0),
                Transform::from_xyz(0.0, 0.0, 1.0),
                PhysicsTranslation::new(Vec3::new(0.0, 0.0, 1.0)),
                Piercing(3),
                PiercingRemaining(0),
            ))
            .id();
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let remaining = app
            .world()
            .get::<PiercingRemaining>(bolt_id)
            .expect("bolt should have PiercingRemaining");
        assert_eq!(
            remaining.0, 3,
            "PiercingRemaining should be reset to Piercing(3), got {}",
            remaining.0
        );
    }

    #[test]
    fn reset_bolt_does_not_touch_chip_effect_components() {
        let mut app = test_app();
        let bolt_id = app
            .world_mut()
            .spawn((
                Bolt,
                BoltVelocity::new(0.0, 0.0),
                Transform::from_xyz(0.0, 0.0, 1.0),
                PhysicsTranslation::new(Vec3::new(0.0, 0.0, 1.0)),
                Piercing(3),
                DamageBoost(0.5),
                BoltSpeedBoost(100.0),
                BoltSizeBoost(2.0),
                ChainHit(1),
            ))
            .id();
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let world = app.world();
        assert_eq!(
            world.get::<Piercing>(bolt_id),
            Some(&Piercing(3)),
            "Piercing should be unchanged"
        );
        assert_eq!(
            world.get::<DamageBoost>(bolt_id),
            Some(&DamageBoost(0.5)),
            "DamageBoost should be unchanged"
        );
        assert_eq!(
            world.get::<BoltSpeedBoost>(bolt_id),
            Some(&BoltSpeedBoost(100.0)),
            "BoltSpeedBoost should be unchanged"
        );
        assert_eq!(
            world.get::<BoltSizeBoost>(bolt_id),
            Some(&BoltSizeBoost(2.0)),
            "BoltSizeBoost should be unchanged"
        );
        assert_eq!(
            world.get::<ChainHit>(bolt_id),
            Some(&ChainHit(1)),
            "ChainHit should be unchanged"
        );
    }

    #[test]
    fn reset_bolt_snaps_physics_translation() {
        let mut app = test_app();
        let bolt_pos = Vec3::new(150.0, 100.0, 1.0);
        spawn_bolt_entity(&mut app, bolt_pos, BoltVelocity::new(0.0, 0.0));
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let config = BoltConfig::default();
        let expected_pos = Vec3::new(0.0, -250.0 + config.spawn_offset_y, 1.0);

        let physics = app
            .world_mut()
            .query_filtered::<&PhysicsTranslation, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have PhysicsTranslation");

        assert_eq!(
            physics.current, expected_pos,
            "PhysicsTranslation.current should be {expected_pos:?}, got {:?}",
            physics.current
        );
        assert_eq!(
            physics.previous, expected_pos,
            "PhysicsTranslation.previous should be {expected_pos:?}, got {:?}",
            physics.previous
        );
    }

    #[test]
    fn reset_bolt_is_noop_when_no_bolt_exists() {
        let mut app = test_app();
        spawn_breaker(&mut app, 0.0, -250.0);

        // Should not panic
        app.update();

        let bolt_count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(bolt_count, 0, "no bolt should be created by reset");
    }

    #[test]
    fn reset_bolt_ignores_extra_bolt_entities() {
        let mut app = test_app();
        // Baseline bolt
        let baseline_pos = Vec3::new(150.0, 100.0, 1.0);
        let baseline_id =
            spawn_bolt_entity(&mut app, baseline_pos, BoltVelocity::new(300.0, 400.0));

        // Extra bolt at a different position
        let extra_pos = Vec3::new(-100.0, 50.0, 1.0);
        let extra_id = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                BoltVelocity::new(200.0, 300.0),
                Transform::from_translation(extra_pos),
                PhysicsTranslation::new(extra_pos),
            ))
            .id();
        spawn_breaker(&mut app, 0.0, -250.0);

        app.update();

        let config = BoltConfig::default();
        let expected_y = -250.0 + config.spawn_offset_y;

        // Baseline should be repositioned
        let baseline_tf = app.world().get::<Transform>(baseline_id).unwrap();
        assert!(
            (baseline_tf.translation.y - expected_y).abs() < f32::EPSILON,
            "baseline bolt should be repositioned to y={expected_y}, got y={}",
            baseline_tf.translation.y
        );

        // Extra bolt should remain untouched
        let extra_tf = app.world().get::<Transform>(extra_id).unwrap();
        assert!(
            (extra_tf.translation.x - extra_pos.x).abs() < f32::EPSILON,
            "extra bolt x should be unchanged at {}, got {}",
            extra_pos.x,
            extra_tf.translation.x
        );
        assert!(
            (extra_tf.translation.y - extra_pos.y).abs() < f32::EPSILON,
            "extra bolt y should be unchanged at {}, got {}",
            extra_pos.y,
            extra_tf.translation.y
        );
    }
}
