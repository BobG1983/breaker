//! System to apply bump grade velocity multiplier to the bolt.

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltVelocity},
    breaker::{
        components::{Breaker, BumpPerfectMultiplier, BumpWeakMultiplier},
        messages::{BumpGrade, BumpPerformed},
    },
};

/// Reads [`BumpPerformed`] messages and applies the corresponding velocity
/// multiplier to the bolt.
///
/// This keeps bolt velocity mutations within the bolt domain, while the
/// breaker domain only grades the bump timing.
pub fn apply_bump_velocity(
    mut reader: MessageReader<BumpPerformed>,
    mut bolt_query: Query<(&mut BoltVelocity, &BoltBaseSpeed, &BoltMaxSpeed), With<Bolt>>,
    breaker_query: Query<(&BumpPerfectMultiplier, &BumpWeakMultiplier), With<Breaker>>,
) {
    // Collect messages first — skip breaker query when there are none (common case).
    let messages: Vec<_> = reader.read().collect();
    if messages.is_empty() {
        return;
    }

    let Ok((perfect_mult, weak_mult)) = breaker_query.single() else {
        return;
    };

    for performed in &messages {
        let multiplier = match performed.grade {
            BumpGrade::Perfect => perfect_mult.0,
            BumpGrade::Early | BumpGrade::Late => weak_mult.0,
        };

        for (mut bolt_velocity, base_speed, max_speed) in &mut bolt_query {
            bolt_velocity.value *= multiplier;

            // Never drop below base speed
            let speed = bolt_velocity.speed();
            if speed < base_speed.0 {
                bolt_velocity.value = bolt_velocity.direction() * base_speed.0;
            }

            // Clamp to max speed
            let speed = bolt_velocity.speed();
            if speed > max_speed.0 {
                bolt_velocity.value = bolt_velocity.direction() * max_speed.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bolt::{
            BoltConfig,
            components::{Bolt, BoltBaseSpeed, BoltRadius, BoltVelocity},
        },
        breaker::resources::BreakerConfig,
    };

    #[derive(Resource)]
    struct TestMessage(Option<BumpPerformed>);

    /// Helper system to queue a message from a test resource.
    fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn spawn_breaker(app: &mut App) {
        let config = BreakerConfig::default();
        app.world_mut().spawn((
            Breaker,
            BumpPerfectMultiplier(config.perfect_bump_multiplier),
            BumpWeakMultiplier(config.weak_bump_multiplier),
        ));
    }

    fn spawn_test_bolt(app: &mut App, vx: f32, vy: f32) {
        let bolt_config = BoltConfig::default();
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(vx, vy),
            BoltBaseSpeed(bolt_config.base_speed),
            BoltMaxSpeed(bolt_config.max_speed),
        ));
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<BumpPerformed>();
        app.add_systems(FixedUpdate, apply_bump_velocity);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn perfect_bump_amplifies_velocity() {
        let mut app = test_app();
        let config = BreakerConfig::default();
        spawn_breaker(&mut app);
        spawn_test_bolt(&mut app, 0.0, 400.0);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(
            FixedUpdate,
            enqueue_from_resource.before(apply_bump_velocity),
        );
        tick(&mut app);

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
        spawn_breaker(&mut app);
        spawn_test_bolt(&mut app, 0.0, bolt_config.max_speed);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(
            FixedUpdate,
            enqueue_from_resource.before(apply_bump_velocity),
        );
        tick(&mut app);

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
        spawn_breaker(&mut app);
        spawn_test_bolt(&mut app, 0.0, 300.0);
        spawn_test_bolt(&mut app, 200.0, 200.0);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        app.add_systems(
            FixedUpdate,
            enqueue_from_resource.before(apply_bump_velocity),
        );
        tick(&mut app);

        let speeds: Vec<f32> = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .map(crate::bolt::components::BoltVelocity::speed)
            .collect();

        assert_eq!(speeds.len(), 2, "both bolts should exist");
        let expected_a = 300.0 * config.perfect_bump_multiplier;
        let expected_b = BoltVelocity::new(200.0, 200.0).speed() * config.perfect_bump_multiplier;
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
            breaker::components::{
                Breaker, BreakerHeight, BreakerTilt, BreakerWidth, MaxReflectionAngle,
                MinAngleFromHorizontal,
            },
            physics::{messages::BoltHitBreaker, systems::bolt_breaker_collision},
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<BumpPerformed>();
        app.add_message::<BoltHitBreaker>();

        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource,
                bolt_breaker_collision,
                apply_bump_velocity.after(bolt_breaker_collision),
            ),
        );

        let breaker_config = BreakerConfig::default();
        let bolt_config = BoltConfig::default();

        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            BreakerWidth(breaker_config.width),
            BreakerHeight(breaker_config.height),
            MaxReflectionAngle(breaker_config.max_reflection_angle.to_radians()),
            MinAngleFromHorizontal(breaker_config.min_angle_from_horizontal.to_radians()),
            BumpPerfectMultiplier(breaker_config.perfect_bump_multiplier),
            BumpWeakMultiplier(breaker_config.weak_bump_multiplier),
            Transform::from_xyz(0.0, breaker_config.y_position, 0.0),
        ));

        let start_y =
            breaker_config.y_position + breaker_config.height / 2.0 + bolt_config.radius + 3.0;
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, -bolt_config.base_speed),
            BoltBaseSpeed(bolt_config.base_speed),
            BoltRadius(bolt_config.radius),
            BoltMaxSpeed(bolt_config.max_speed),
            Transform::from_xyz(0.0, start_y, 0.0),
        ));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();

        assert!(vel.value.y > 0.0, "bolt should reflect upward");
        let post_collision_speed = bolt_config.base_speed;
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
        let bolt_config = BoltConfig::default();
        spawn_breaker(&mut app);
        // Start above base speed so weak multiplier has room to reduce
        let start_speed = 600.0;
        assert!(start_speed * config.weak_bump_multiplier > bolt_config.base_speed);
        spawn_test_bolt(&mut app, 0.0, start_speed);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
        })));

        app.add_systems(
            FixedUpdate,
            enqueue_from_resource.before(apply_bump_velocity),
        );
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = start_speed * config.weak_bump_multiplier;
        assert!(
            (vel.value.y - expected).abs() < 1.0,
            "early bump should apply weak multiplier"
        );
    }

    #[test]
    fn weak_bump_never_drops_below_base_speed() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        spawn_breaker(&mut app);
        // Bolt already at base speed
        spawn_test_bolt(&mut app, 0.0, bolt_config.base_speed);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
        })));

        app.add_systems(
            FixedUpdate,
            enqueue_from_resource.before(apply_bump_velocity),
        );
        tick(&mut app);

        let vel = app
            .world_mut()
            .query::<&BoltVelocity>()
            .iter(app.world())
            .next()
            .unwrap();
        assert!(
            vel.speed() >= bolt_config.base_speed - f32::EPSILON,
            "weak bump should not drop speed below base_speed ({:.0}), got {:.0}",
            bolt_config.base_speed,
            vel.speed()
        );
    }
}
