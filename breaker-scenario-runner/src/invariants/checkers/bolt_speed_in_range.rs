use bevy::prelude::*;
use breaker::bolt::components::{BoltMaxSpeed, BoltMinSpeed, BoltVelocity};

use crate::{invariants::*, types::InvariantKind};

/// Checks that bolt speed stays within configured min/max bounds.
///
/// Skips bolts with zero speed (serving or dead bolts).
pub fn check_bolt_speed_in_range(
    bolts: Query<(Entity, &BoltVelocity, &BoltMinSpeed, &BoltMaxSpeed), With<ScenarioTagBolt>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    const SPEED_TOLERANCE: f32 = 1.0;
    for (entity, velocity, min_speed, max_speed) in &bolts {
        let speed = velocity.speed();
        if speed < f32::EPSILON {
            continue;
        }
        if speed < min_speed.0 - SPEED_TOLERANCE || speed > max_speed.0 + SPEED_TOLERANCE {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltSpeedInRange,
                entity: Some(entity),
                message: format!(
                    "BoltSpeedInRange FAIL frame={} entity={entity:?} speed={speed:.1} bounds=[{:.1}, {:.1}]",
                    frame.0, min_speed.0, max_speed.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_bolt_speed() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_bolt_speed_in_range);
        app
    }

    #[test]
    fn bolt_speed_in_range_fires_when_above_max() {
        let mut app = test_app_bolt_speed();

        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 1000.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltSpeedInRange);
    }

    #[test]
    fn bolt_speed_in_range_does_not_fire_when_within_bounds() {
        let mut app = test_app_bolt_speed();

        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 400.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn bolt_speed_in_range_skips_zero_speed() {
        let mut app = test_app_bolt_speed();

        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 0.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "zero speed should be skipped");
    }

    /// Bolt speed 800.5 with max=800.0 is within 1.0 tolerance — no violation.
    #[test]
    fn bolt_speed_in_range_does_not_fire_when_speed_is_slightly_above_max_within_tolerance() {
        let mut app = test_app_bolt_speed();

        // speed() = Vec2::new(0.0, 800.5).length() = 800.5
        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 800.5),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::BoltSpeedInRange),
            "expected no BoltSpeedInRange violation for speed=800.5 with max=800.0 \
            (within 1.0 tolerance), got: {:?}",
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::BoltSpeedInRange)
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
    }

    /// Bolt speed 802.0 with max=800.0 exceeds tolerance of 1.0 → violation fires.
    #[test]
    fn bolt_speed_in_range_fires_when_speed_is_well_above_max_beyond_tolerance() {
        let mut app = test_app_bolt_speed();

        // speed() = Vec2::new(0.0, 802.0).length() = 802.0
        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 802.0),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::BoltSpeedInRange)
                .count(),
            1,
            "expected exactly 1 BoltSpeedInRange violation for speed=802.0 with max=800.0 \
            (exceeds 1.0 tolerance), got: {:?}",
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::BoltSpeedInRange)
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
    }

    /// Bolt speed 199.5 with min=200.0 is within 1.0 tolerance — no violation.
    #[test]
    fn bolt_speed_in_range_does_not_fire_when_speed_is_slightly_below_min_within_tolerance() {
        let mut app = test_app_bolt_speed();

        // speed() = Vec2::new(0.0, 199.5).length() = 199.5
        app.world_mut().spawn((
            ScenarioTagBolt,
            BoltVelocity::new(0.0, 199.5),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::BoltSpeedInRange),
            "expected no BoltSpeedInRange violation for speed=199.5 with min=200.0 \
            (within 1.0 tolerance), got: {:?}",
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::BoltSpeedInRange)
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
    }
}
