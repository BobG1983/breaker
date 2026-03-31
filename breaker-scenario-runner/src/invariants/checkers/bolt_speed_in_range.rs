use bevy::prelude::*;
use breaker::{
    bolt::components::{BoltMaxSpeed, BoltMinSpeed},
    effect::effects::speed_boost::ActiveSpeedBoosts,
};
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{invariants::*, types::InvariantKind};

type BoltSpeedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Velocity2D,
        &'static BoltMinSpeed,
        &'static BoltMaxSpeed,
        Option<&'static ActiveSpeedBoosts>,
    ),
    With<ScenarioTagBolt>,
>;

/// Checks that bolt speed stays within configured min/max bounds.
///
/// Reads speed from `Velocity2D`. Computes effective bounds from
/// `ActiveSpeedBoosts` (same source as `prepare_bolt_velocity`).
/// Skips bolts with zero speed (serving or dead bolts).
pub fn check_bolt_speed_in_range(
    bolts: BoltSpeedQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    const SPEED_TOLERANCE: f32 = 1.0;
    for (entity, velocity, min_speed, max_speed, active_boosts) in &bolts {
        let speed = velocity.speed();
        if speed < f32::EPSILON {
            continue;
        }
        let mult = active_boosts.map_or(1.0, ActiveSpeedBoosts::multiplier);
        let effective_min = min_speed.0 * mult;
        let effective_max = max_speed.0 * mult;
        if speed < effective_min - SPEED_TOLERANCE || speed > effective_max + SPEED_TOLERANCE {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltSpeedInRange,
                entity: Some(entity),
                message: format!(
                    "BoltSpeedInRange FAIL frame={} entity={entity:?} speed={speed:.1} bounds=[{:.1}, {:.1}] mult={mult:.2}",
                    frame.0, effective_min, effective_max,
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
            Velocity2D(Vec2::new(0.0, 1000.0)),
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
            Velocity2D(Vec2::new(0.0, 400.0)),
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
            Velocity2D(Vec2::new(0.0, 0.0)),
            BoltMinSpeed(200.0),
            BoltMaxSpeed(800.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "zero speed should be skipped");
    }

    /// Bolt speed 800.5 with max=800.0 is within 1.0 tolerance -- no violation.
    #[test]
    fn bolt_speed_in_range_does_not_fire_when_speed_is_slightly_above_max_within_tolerance() {
        let mut app = test_app_bolt_speed();

        // speed() = Vec2::new(0.0, 800.5).length() = 800.5
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 800.5)),
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

    /// Bolt speed 802.0 with max=800.0 exceeds tolerance of 1.0 -- violation fires.
    #[test]
    fn bolt_speed_in_range_fires_when_speed_is_well_above_max_beyond_tolerance() {
        let mut app = test_app_bolt_speed();

        // speed() = Vec2::new(0.0, 802.0).length() = 802.0
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 802.0)),
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

    /// Bolt speed 199.5 with min=200.0 is within 1.0 tolerance -- no violation.
    #[test]
    fn bolt_speed_in_range_does_not_fire_when_speed_is_slightly_below_min_within_tolerance() {
        let mut app = test_app_bolt_speed();

        // speed() = Vec2::new(0.0, 199.5).length() = 199.5
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 199.5)),
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

    // ── Velocity2D migration tests ────────────────────────────────

    /// Bolt with `Velocity2D`(0.0, 1000.0), min=200, max=800 -- speed 1000 exceeds max.
    /// `check_bolt_speed_in_range` should detect this via `Velocity2D`.
    #[test]
    fn bolt_speed_in_range_reads_velocity2d_fires_when_above_max() {
        use rantzsoft_spatial2d::components::Velocity2D;

        let mut app = test_app_bolt_speed();

        // Spawn with Velocity2D only to prove the system reads Velocity2D
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 1000.0)),
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
            "expected 1 BoltSpeedInRange violation for Velocity2D speed=1000 with max=800, got: {:?}",
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::BoltSpeedInRange)
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
    }
}
