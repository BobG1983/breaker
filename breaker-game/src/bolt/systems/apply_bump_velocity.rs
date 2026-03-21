//! System to apply bump grade velocity multiplier to the bolt.

use bevy::prelude::*;

use crate::{
    bolt::components::*, breaker::messages::BumpPerformed, chips::components::BoltSpeedBoost,
};

/// Reads [`BumpPerformed`] messages and applies the velocity multiplier to the bolt.
///
/// The multiplier is included in the message by the breaker domain, eliminating
/// the need for cross-domain component reads.
pub(crate) fn apply_bump_velocity(
    mut reader: MessageReader<BumpPerformed>,
    mut bolt_query: Query<
        (
            &mut BoltVelocity,
            &BoltBaseSpeed,
            &BoltMaxSpeed,
            Option<&BoltSpeedBoost>,
        ),
        With<Bolt>,
    >,
) {
    // One BumpPerformed per tick is the invariant (one bump action per fixed step).
    // Take the first message and drain any extras to prevent compounded velocity
    // multiplication if a duplicate is ever emitted in the same frame.
    let mut messages = reader.read();
    let Some(performed) = messages.next() else {
        return;
    };

    let multiplier = performed.multiplier;

    for (mut bolt_velocity, base_speed, max_speed, speed_boost) in &mut bolt_query {
        let boost = speed_boost.map_or(0.0, |b| b.0);

        bolt_velocity.value *= multiplier;

        // Never drop below effective base speed (base + boost)
        let speed = bolt_velocity.speed();
        if speed < base_speed.0 + boost {
            bolt_velocity.value = bolt_velocity.direction() * (base_speed.0 + boost);
        }

        // Clamp to effective max speed (max + boost)
        let speed = bolt_velocity.speed();
        if speed > max_speed.0 + boost {
            bolt_velocity.value = bolt_velocity.direction() * (max_speed.0 + boost);
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
        breaker::messages::BumpGrade,
    };

    #[derive(Resource)]
    struct TestMessage(Option<BumpPerformed>);

    /// Helper system to queue a message from a test resource.
    fn enqueue_from_resource(msg_res: Res<TestMessage>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    const TEST_PERFECT_MULT: f32 = 1.5;
    const TEST_WEAK_MULT: f32 = 1.1;

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
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_systems(FixedUpdate, apply_bump_velocity);
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
        spawn_test_bolt(&mut app, 0.0, 400.0);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: TEST_PERFECT_MULT,
            bolt: Entity::PLACEHOLDER,
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
        let expected = 400.0 * TEST_PERFECT_MULT;
        assert!(
            (vel.value.y - expected).abs() < 1.0,
            "perfect bump should amplify velocity"
        );
    }

    #[test]
    fn perfect_bump_on_max_speed_bolt_is_clamped() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        spawn_test_bolt(&mut app, 0.0, bolt_config.max_speed);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: TEST_PERFECT_MULT,
            bolt: Entity::PLACEHOLDER,
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
        let unclamped = bolt_config.max_speed * TEST_PERFECT_MULT;
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
        spawn_test_bolt(&mut app, 0.0, 300.0);
        spawn_test_bolt(&mut app, 200.0, 200.0);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: TEST_PERFECT_MULT,
            bolt: Entity::PLACEHOLDER,
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
        let expected_a = 300.0 * TEST_PERFECT_MULT;
        let expected_b = BoltVelocity::new(200.0, 200.0).speed() * TEST_PERFECT_MULT;
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
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitBreaker>();

        app.add_systems(
            FixedUpdate,
            (
                enqueue_from_resource,
                bolt_breaker_collision,
                apply_bump_velocity.after(bolt_breaker_collision),
            ),
        );

        let breaker_config = crate::breaker::BreakerConfig::default();
        let bolt_config = BoltConfig::default();

        app.world_mut().spawn((
            Breaker,
            BreakerTilt::default(),
            BreakerWidth(breaker_config.width),
            BreakerHeight(breaker_config.height),
            MaxReflectionAngle(breaker_config.max_reflection_angle.to_radians()),
            MinAngleFromHorizontal(breaker_config.min_angle_from_horizontal.to_radians()),
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
            multiplier: TEST_PERFECT_MULT,
            bolt: Entity::PLACEHOLDER,
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
        let expected_min = post_collision_speed * TEST_PERFECT_MULT * 0.9;
        assert!(
            vel.speed() >= expected_min,
            "bump multiplier should survive collision — speed {:.0} should be >= {expected_min:.0}",
            vel.speed(),
        );
    }

    #[test]
    fn weak_bump_amplifies_velocity() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        // Start above base speed so weak multiplier has room to reduce
        let start_speed = 600.0;
        assert!(start_speed * TEST_WEAK_MULT > bolt_config.base_speed);
        spawn_test_bolt(&mut app, 0.0, start_speed);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
            multiplier: TEST_WEAK_MULT,
            bolt: Entity::PLACEHOLDER,
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
        let expected = start_speed * TEST_WEAK_MULT;
        assert!(
            (vel.value.y - expected).abs() < 1.0,
            "early bump should apply weak multiplier"
        );
    }

    #[test]
    fn identity_multiplier_leaves_velocity_unchanged() {
        let mut app = test_app();
        spawn_test_bolt(&mut app, 0.0, 400.0);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: 1.0,
            bolt: Entity::PLACEHOLDER,
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
            (vel.value.y - 400.0).abs() < 1.0,
            "identity multiplier should leave velocity unchanged"
        );
    }

    #[test]
    fn weak_bump_never_drops_below_base_speed() {
        let mut app = test_app();
        let bolt_config = BoltConfig::default();
        // Bolt already at base speed
        spawn_test_bolt(&mut app, 0.0, bolt_config.base_speed);

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
            multiplier: TEST_WEAK_MULT,
            bolt: Entity::PLACEHOLDER,
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

    #[test]
    fn bolt_with_speed_boost_clamps_to_elevated_max() {
        use crate::chips::components::BoltSpeedBoost;

        let mut app = test_app();
        // Bolt at max speed with BoltSpeedBoost(100.0): effective max = 800 + 100 = 900
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 800.0),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            BoltSpeedBoost(100.0),
        ));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: TEST_PERFECT_MULT,
            bolt: Entity::PLACEHOLDER,
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
        // 800 * 1.5 = 1200, should clamp to effective max 900 (800 + 100), not 800
        let effective_max = 800.0 + 100.0;
        assert!(
            (vel.speed() - effective_max).abs() < 1.0,
            "speed {:.1} should be clamped to elevated max {effective_max:.1} (base max + boost), \
             not raw max 800.0",
            vel.speed(),
        );
    }

    #[test]
    fn bolt_with_speed_boost_floors_at_elevated_base() {
        use crate::chips::components::BoltSpeedBoost;

        let mut app = test_app();
        // Bolt at base speed with BoltSpeedBoost(100.0): effective base = 400 + 100 = 500
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            BoltSpeedBoost(100.0),
        ));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
            multiplier: 0.5,
            bolt: Entity::PLACEHOLDER,
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
        // 400 * 0.5 = 200, should floor at effective base 500 (400 + 100), not 400
        let effective_base = 400.0 + 100.0;
        assert!(
            (vel.speed() - effective_base).abs() < 1.0,
            "speed {:.1} should be floored at elevated base {effective_base:.1} \
             (base speed + boost), not raw base 400.0",
            vel.speed(),
        );
    }

    #[test]
    fn sub_one_multiplier_floors_at_base_speed() {
        let mut app = test_app();
        // Spawn bolt at base speed with explicit components (no BoltSpeedBoost)
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
        ));

        // Sub-1.0 multiplier: 400.0 * 0.5 = 200.0, which is below base_speed
        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Early,
            multiplier: 0.5,
            bolt: Entity::PLACEHOLDER,
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
        // Unclamped would be 200.0; floor guard must clamp to base_speed 400.0
        assert!(
            (vel.speed() - 400.0).abs() < 1.0,
            "sub-1.0 multiplier should floor at base_speed 400.0, not unclamped 200.0; got {:.1}",
            vel.speed(),
        );
        // Direction should be preserved (still pointing up)
        assert!(
            vel.value.y > 0.0,
            "direction should be preserved after floor clamp"
        );
    }

    #[test]
    fn bolt_without_speed_boost_unchanged() {
        let mut app = test_app();
        // No BoltSpeedBoost — existing behavior should be unchanged
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
        ));

        app.insert_resource(TestMessage(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            multiplier: TEST_PERFECT_MULT,
            bolt: Entity::PLACEHOLDER,
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
        // 400 * 1.5 = 600, within max of 800, no boost component
        let expected = 400.0 * TEST_PERFECT_MULT;
        assert!(
            (vel.speed() - expected).abs() < 1.0,
            "speed {:.1} should be {expected:.1} (400 * 1.5) — no speed boost component present",
            vel.speed(),
        );
    }
}
