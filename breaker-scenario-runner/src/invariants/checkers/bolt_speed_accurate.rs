use bevy::prelude::*;
use breaker::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};
use rantzsoft_spatial2d::components::{BaseSpeed, MaxSpeed, MinSpeed, Velocity2D};

use crate::{invariants::*, types::InvariantKind};

type BoltSpeedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Velocity2D,
        &'static BaseSpeed,
        &'static MinSpeed,
        &'static MaxSpeed,
        Option<&'static EffectStack<SpeedBoostConfig>>,
    ),
    With<ScenarioTagBolt>,
>;

/// Checks that bolt speed equals the expected derived value.
///
/// Under the velocity model, bolt speed is deterministic:
/// `(base_speed * mult).clamp(min_speed, max_speed)`.
/// Every system that modifies velocity applies this formula, so
/// the actual speed should always match the expected value within
/// floating-point tolerance.
///
/// Skips bolts with zero speed (serving or dead bolts).
pub fn check_bolt_speed_accurate(
    bolts: BoltSpeedQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    const SPEED_TOLERANCE: f32 = 1.0;
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    for (entity, velocity, base_speed, min_speed, max_speed, active_boosts) in &bolts {
        let speed = velocity.speed();
        if speed < f32::EPSILON {
            continue;
        }
        let mult = active_boosts.map_or(1.0, EffectStack::aggregate);
        let expected = (base_speed.0 * mult).clamp(min_speed.0, max_speed.0);
        if (speed - expected).abs() > SPEED_TOLERANCE {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltSpeedAccurate,
                entity: Some(entity),
                message: format!(
                    "BoltSpeedAccurate FAIL frame={} entity={entity:?} speed={speed:.1} expected={expected:.1} base={:.1} mult={mult:.2} bounds=[{:.1}, {:.1}]",
                    frame.0, base_speed.0, min_speed.0, max_speed.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

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
            .add_systems(FixedUpdate, check_bolt_speed_accurate);
        app
    }

    fn speed_stack(values: &[f32]) -> EffectStack<SpeedBoostConfig> {
        let mut stack = EffectStack::default();
        for &v in values {
            stack.push(
                String::new(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(v),
                },
            );
        }
        stack
    }

    #[test]
    fn no_violation_when_speed_matches_expected() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn fires_when_speed_does_not_match_expected() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 600.0)),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BoltSpeedAccurate);
    }

    #[test]
    fn no_violation_within_tolerance() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 400.5)),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn skips_zero_speed() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::ZERO),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn no_violation_with_speed_boost_at_expected() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
            speed_stack(&[2.0]),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn fires_with_speed_boost_when_speed_wrong() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
            speed_stack(&[2.0]),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
    }

    #[test]
    fn base_below_min_clamps_to_min() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 200.0)),
            BaseSpeed(100.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn base_above_max_clamps_to_max() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BaseSpeed(1000.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }

    #[test]
    fn boost_above_max_clamps_to_max() {
        let mut app = test_app_bolt_speed();
        app.world_mut().spawn((
            ScenarioTagBolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BaseSpeed(400.0),
            MinSpeed(200.0),
            MaxSpeed(800.0),
            speed_stack(&[3.0]),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }
}
