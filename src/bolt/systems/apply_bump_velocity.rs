//! System to apply bump grade velocity multiplier to the bolt.

use bevy::prelude::*;

use crate::{
    bolt::{
        BoltConfig,
        components::{Bolt, BoltVelocity},
    },
    breaker::{
        BreakerConfig,
        messages::{BumpGrade, BumpPerformed},
    },
};

/// Reads [`BumpPerformed`] messages and applies the corresponding velocity
/// multiplier to the bolt.
///
/// This keeps bolt velocity mutations within the bolt domain, while the
/// breaker domain only grades the bump timing.
pub fn apply_bump_velocity(
    bolt_config: Res<BoltConfig>,
    config: Res<BreakerConfig>,
    mut reader: MessageReader<BumpPerformed>,
    mut bolt_query: Query<&mut BoltVelocity, With<Bolt>>,
) {
    for performed in reader.read() {
        let multiplier = match performed.grade {
            BumpGrade::Perfect => config.perfect_bump_multiplier,
            BumpGrade::Early | BumpGrade::Late => config.weak_bump_multiplier,
        };

        for mut bolt_velocity in &mut bolt_query {
            bolt_velocity.value *= multiplier;

            // Clamp to max speed
            let speed = bolt_velocity.speed();
            if speed > bolt_config.max_speed {
                bolt_velocity.value = bolt_velocity.direction() * bolt_config.max_speed;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::{
        BoltConfig,
        components::{Bolt, BoltVelocity},
    };

    #[derive(Resource)]
    struct TestMessage(Option<BumpPerformed>);

    /// Helper system to queue a message from a test resource.
    fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<BreakerConfig>();
        app.add_message::<BumpPerformed>();
        app.add_systems(Update, apply_bump_velocity);
        app
    }

    #[test]
    fn perfect_bump_amplifies_velocity() {
        let mut app = test_app();
        let config = BreakerConfig::default();

        app.world_mut().spawn((Bolt, BoltVelocity::new(0.0, 400.0)));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(Update, enqueue_from_resource.before(apply_bump_velocity));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = 400.0 * config.perfect_bump_multiplier;
        assert!(
            (vel.value.y - expected).abs() < 1.0,
            "perfect bump should amplify velocity"
        );
    }

    #[test]
    fn perfect_bump_on_max_speed_bolt_is_clamped() {
        let mut app = test_app();
        let breaker_config = BreakerConfig::default();
        let bolt_config = BoltConfig::default();

        // Bolt already at max speed
        app.world_mut()
            .spawn((Bolt, BoltVelocity::new(0.0, bolt_config.max_speed)));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(Update, enqueue_from_resource.before(apply_bump_velocity));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        let unclamped = bolt_config.max_speed * breaker_config.perfect_bump_multiplier;
        assert!(
            vel.speed() <= bolt_config.max_speed + 1.0,
            "speed {:.0} should be clamped to max_speed {:.0}, not unclamped {unclamped:.0}",
            vel.speed(),
            bolt_config.max_speed,
        );
    }

    #[test]
    fn multiple_bolts_each_get_bump_velocity() {
        let mut app = test_app();
        let config = BreakerConfig::default();

        app.world_mut().spawn((Bolt, BoltVelocity::new(0.0, 300.0)));
        app.world_mut()
            .spawn((Bolt, BoltVelocity::new(200.0, 200.0)));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(Update, enqueue_from_resource.before(apply_bump_velocity));
        app.update();

        let speeds: Vec<f32> = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .map(crate::bolt::components::BoltVelocity::speed)
            .collect();

        assert_eq!(speeds.len(), 2, "both bolts should exist");
        let expected_a = 300.0 * config.perfect_bump_multiplier;
        let expected_b = BoltVelocity::new(200.0, 200.0).speed() * config.perfect_bump_multiplier;
        // At least one bolt should match each expected speed (order not guaranteed)
        assert!(
            speeds.iter().any(|s| (*s - expected_a).abs() < 1.0),
            "first bolt speed should be amplified"
        );
        assert!(
            speeds.iter().any(|s| (*s - expected_b).abs() < 1.0),
            "second bolt speed should be amplified"
        );
    }

    #[test]
    fn bump_multiplier_survives_breaker_collision() {
        use crate::{
            breaker::components::{Breaker, BreakerTilt},
            physics::{
                messages::BoltHitBreaker, resources::PhysicsConfig, systems::bolt_breaker_collision,
            },
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<BoltConfig>();
        app.init_resource::<BreakerConfig>();
        app.init_resource::<PhysicsConfig>();
        app.add_message::<BumpPerformed>();
        app.add_message::<BoltHitBreaker>();

        // Run collision first, then bump — matching production ordering.
        app.add_systems(
            Update,
            (
                enqueue_from_resource,
                bolt_breaker_collision,
                apply_bump_velocity.after(bolt_breaker_collision),
            ),
        );

        let breaker_config = BreakerConfig::default();
        let bolt_config = BoltConfig::default();

        // Spawn breaker at default Y position
        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            Transform::from_xyz(0.0, breaker_config.y_position, 0.0),
        ));

        // Place bolt just above breaker, moving downward — will collide this tick
        let start_y =
            breaker_config.y_position + breaker_config.half_height + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -bolt_config.base_speed),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));

        // Queue a perfect bump on the same tick
        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        // Advance fixed time so CCD ray covers the distance
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(timestep);
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();

        // After collision, bolt reflects upward at ~base_speed.
        // After bump, speed should be amplified by perfect_bump_multiplier.
        assert!(vel.value.y > 0.0, "bolt should reflect upward");
        let post_collision_speed = bolt_config.base_speed; // collision resets to at least base_speed
        let expected_min = post_collision_speed * breaker_config.perfect_bump_multiplier * 0.9;
        assert!(
            vel.speed() >= expected_min,
            "bump multiplier should survive collision — speed {:.0} should be >= {expected_min:.0}",
            vel.speed(),
        );
    }

    #[test]
    fn weak_bump_reduces_velocity() {
        let mut app = test_app();
        let config = BreakerConfig::default();

        app.world_mut().spawn((Bolt, BoltVelocity::new(0.0, 400.0)));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
        })));

        app.add_systems(Update, enqueue_from_resource.before(apply_bump_velocity));
        app.update();

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = 400.0 * config.weak_bump_multiplier;
        assert!(
            (vel.value.y - expected).abs() < 1.0,
            "early bump should apply weak multiplier"
        );
    }
}
