use bevy::prelude::*;
use breaker::protocol::protocols::burnout::system::BurnoutHeat;

use crate::{invariants::*, types::InvariantKind};

/// Checks that every [`BurnoutHeat`] component in the world has:
/// - `heat` within `[0.0, 1.0]` (inclusive), and
/// - `still_timer >= 0.0`
///
/// Both bounds are enforced by `burnout_update_heat` at each tick. A drift
/// outside these bounds indicates a bug in the update/reset path — for example,
/// a float accumulation error that overshoots the clamp, a manual write that
/// bypasses the clamp, or a `still_timer` reset that subtracts below zero.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_burnout_heat_clamped(
    heats: Query<(Entity, &BurnoutHeat)>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }

    if let Some(ref stats) = stats
        && !stats.entered_playing
    {
        return;
    }

    for (entity, heat) in &heats {
        if heat.heat < 0.0 || heat.heat > 1.0 {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::BurnoutHeatClamped,
                entity:    Some(entity),
                message:   format!(
                    "BurnoutHeatClamped FAIL frame={} entity={entity:?} heat={} \
                     (expected [0.0, 1.0])",
                    frame.0, heat.heat,
                ),
            });
        }
        if heat.still_timer < 0.0 {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::BurnoutHeatClamped,
                entity:    Some(entity),
                message:   format!(
                    "BurnoutHeatClamped FAIL frame={} entity={entity:?} still_timer={} \
                     (expected >= 0.0)",
                    frame.0, heat.still_timer,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::protocol::protocols::burnout::system::BurnoutHeat;

    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(ScenarioStats {
                entered_playing: true,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_burnout_heat_clamped);
        app
    }

    // -- heat within bounds produces no violation --

    #[test]
    fn heat_within_bounds_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.5,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations for heat=0.5, got {}",
            log.0.len()
        );
    }

    #[test]
    fn heat_at_zero_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.0,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "heat=0.0 should not violate");
    }

    #[test]
    fn heat_at_one_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              1.0,
            still_timer:       0.0,
            mega_bump_charged: true,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty(), "heat=1.0 should not violate");
    }

    // -- heat > 1.0 fires violation --

    #[test]
    fn heat_above_one_fires_violation() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 30;
        app.world_mut().spawn(BurnoutHeat {
            heat:              2.0,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for heat=2.0, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutHeatClamped);
        assert_eq!(log.0[0].frame, 30);
        assert!(
            log.0[0].message.contains("heat=2"),
            "message should contain heat value, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].entity.is_some(),
            "entity should be Some for component-level checker"
        );
    }

    // -- heat < 0.0 fires violation --

    #[test]
    fn heat_below_zero_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              -0.1,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for heat=-0.1, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutHeatClamped);
        assert!(
            log.0[0].message.contains("heat="),
            "message should contain heat value"
        );
    }

    // -- still_timer < 0.0 fires violation --

    #[test]
    fn still_timer_negative_fires_violation() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 15;
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.5,
            still_timer:       -1.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for still_timer=-1.0, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutHeatClamped);
        assert_eq!(log.0[0].frame, 15);
        assert!(
            log.0[0].message.contains("still_timer="),
            "message should contain still_timer value, got: {}",
            log.0[0].message
        );
    }

    // -- both heat out-of-range AND still_timer negative produces two violations --

    #[test]
    fn both_heat_out_of_range_and_negative_still_timer_produce_two_violations() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              1.5,
            still_timer:       -0.5,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for heat=1.5 and still_timer=-0.5, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::BurnoutHeatClamped)
        );
    }

    // -- does not fire when entered_playing is false --

    #[test]
    fn does_not_fire_when_entered_playing_false() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(ScenarioStats {
                entered_playing: false,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_burnout_heat_clamped);
        app.world_mut().spawn(BurnoutHeat {
            heat:              2.0,
            still_timer:       -1.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when entered_playing is false, got {}",
            log.0.len()
        );
    }

    // -- fires when ScenarioStats is absent --

    #[test]
    fn fires_when_scenario_stats_absent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_burnout_heat_clamped);
        app.world_mut().spawn(BurnoutHeat {
            heat:              2.0,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when ScenarioStats absent and heat=2.0, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutHeatClamped);
    }

    // -- increments invariant_checks --

    #[test]
    fn increments_invariant_checks() {
        let mut app = test_app();
        tick(&mut app);
        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.invariant_checks, 1,
            "invariant_checks should be 1 after one invocation, got {}",
            stats.invariant_checks
        );
    }

    // -- no entities with BurnoutHeat produces no violation --

    #[test]
    fn no_burnout_heat_entities_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when no BurnoutHeat entities exist"
        );
    }

    // -- violation message contains frame number --

    #[test]
    fn violation_message_contains_frame_number() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 77;
        app.world_mut().spawn(BurnoutHeat {
            heat:              -0.5,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=77"),
            "message should contain frame=77, got: {}",
            log.0[0].message
        );
    }
}
