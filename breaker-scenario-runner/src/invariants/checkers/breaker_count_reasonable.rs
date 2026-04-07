use bevy::prelude::*;
use breaker::breaker::components::PrimaryBreaker;

use crate::{invariants::*, types::InvariantKind};

/// Checks that exactly one [`PrimaryBreaker`] entity exists during gameplay.
///
/// Unlike bolt/pulse/chain/gravity checkers that use configurable maxima from
/// `InvariantParams`, this checker hardcodes the expected count to exactly 1.
///
/// Gated on [`ScenarioStats::entered_playing`]: when [`ScenarioStats`] is present
/// and `entered_playing` is `false`, the checker early-returns without producing
/// violations. This prevents false positives during loading states.
///
/// Increments [`ScenarioStats::invariant_checks`] when the check runs.
pub fn check_breaker_count_reasonable(
    breakers: Query<Entity, With<PrimaryBreaker>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }

    // Gate: do not check invariants until the game has entered Playing.
    // When ScenarioStats is present but entered_playing is false, we are
    // still in Loading/MainMenu — entities may not be fully initialized.
    if let Some(ref stats) = stats
        && !stats.entered_playing
    {
        return;
    }

    let count = breakers.iter().count();
    if count != 1 {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::BreakerCountReasonable,
            entity: None,
            message: format!(
                "BreakerCountReasonable FAIL frame={} count={count}",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::breaker::components::ExtraBreaker;

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
            .add_systems(FixedUpdate, check_breaker_count_reasonable);
        app
    }

    // -- Behavior 6: Zero PrimaryBreaker entities during Playing fires violation --

    #[test]
    fn zero_primary_breakers_during_playing_fires_violation() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 10;
        // No PrimaryBreaker entities spawned
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for 0 PrimaryBreaker entities, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerCountReasonable);
        assert_eq!(log.0[0].frame, 10);
        assert!(
            log.0[0].entity.is_none(),
            "entity should be None for count-based checker"
        );
        assert!(
            log.0[0].message.contains("count=0"),
            "violation message should contain 'count=0', got: {}",
            log.0[0].message
        );
    }

    // -- Behavior 7: Two PrimaryBreaker entities during Playing fires violation --

    #[test]
    fn two_primary_breakers_during_playing_fires_violation() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 25;
        app.world_mut().spawn(PrimaryBreaker);
        app.world_mut().spawn(PrimaryBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for 2 PrimaryBreaker entities, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerCountReasonable);
        assert_eq!(log.0[0].frame, 25);
        assert!(
            log.0[0].message.contains("count=2"),
            "violation message should contain 'count=2', got: {}",
            log.0[0].message
        );
    }

    #[test]
    fn three_primary_breakers_fires_exactly_one_violation() {
        let mut app = test_app();
        for _ in 0..3 {
            app.world_mut().spawn(PrimaryBreaker);
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation for 3 PrimaryBreaker entities, got {}",
            log.0.len()
        );
        assert!(
            log.0[0].message.contains("count=3"),
            "violation message should contain 'count=3', got: {}",
            log.0[0].message
        );
    }

    // -- Behavior 8: Exactly one PrimaryBreaker during Playing produces no violation --

    #[test]
    fn exactly_one_primary_breaker_during_playing_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(PrimaryBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when exactly 1 PrimaryBreaker exists, got {}",
            log.0.len()
        );
    }

    #[test]
    fn one_primary_breaker_with_unrelated_entity_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(PrimaryBreaker);
        // Unrelated entity should not be counted
        app.world_mut().spawn(Transform::default());
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when 1 PrimaryBreaker + unrelated entity exist, got {}",
            log.0.len()
        );
    }

    // -- Behavior 9: Checker does not fire when entered_playing is false (zero breakers) --

    #[test]
    fn does_not_fire_when_entered_playing_false_zero_breakers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame(5))
            .insert_resource(ScenarioStats {
                entered_playing: false,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_breaker_count_reasonable);
        // No PrimaryBreaker entities spawned
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when entered_playing is false, got {}",
            log.0.len()
        );
    }

    // -- Behavior 10: Checker does not fire when entered_playing is false (two breakers) --

    #[test]
    fn does_not_fire_when_entered_playing_false_two_breakers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame(5))
            .insert_resource(ScenarioStats {
                entered_playing: false,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_breaker_count_reasonable);
        app.world_mut().spawn(PrimaryBreaker);
        app.world_mut().spawn(PrimaryBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when entered_playing is false (2 breakers), got {}",
            log.0.len()
        );
    }

    // -- Behavior 11: Checker fires when ScenarioStats resource is absent --

    #[test]
    fn fires_when_scenario_stats_absent_and_zero_breakers() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame(5))
            // Deliberately NOT inserting ScenarioStats
            .add_systems(FixedUpdate, check_breaker_count_reasonable);
        // No PrimaryBreaker entities
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when ScenarioStats is absent and 0 breakers, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerCountReasonable);
    }

    #[test]
    fn no_violation_when_scenario_stats_absent_and_one_breaker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame(5))
            // Deliberately NOT inserting ScenarioStats
            .add_systems(FixedUpdate, check_breaker_count_reasonable);
        app.world_mut().spawn(PrimaryBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when ScenarioStats is absent and 1 breaker, got {}",
            log.0.len()
        );
    }

    // -- Behavior 12: Violation message includes frame number --

    #[test]
    fn violation_message_includes_frame_number() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 42;
        // No PrimaryBreaker entities
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation, got {}",
            log.0.len()
        );
        assert!(
            log.0[0].message.contains("frame=42"),
            "violation message should contain 'frame=42', got: {}",
            log.0[0].message
        );
    }

    // -- Behavior 13: Violation entity field is None --

    #[test]
    fn violation_entity_field_is_none() {
        let mut app = test_app();
        app.world_mut().spawn(PrimaryBreaker);
        app.world_mut().spawn(PrimaryBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 violation, got {}",
            log.0.len()
        );
        assert!(
            log.0[0].entity.is_none(),
            "violation entity should be None for count-based checker, got: {:?}",
            log.0[0].entity
        );
    }

    // -- Behavior 14: Checker increments ScenarioStats::invariant_checks --

    #[test]
    fn increments_invariant_checks_on_normal_run() {
        let mut app = test_app();
        app.world_mut().spawn(PrimaryBreaker);
        tick(&mut app);
        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.invariant_checks, 1,
            "invariant_checks should be 1 after one checker invocation, got {}",
            stats.invariant_checks
        );
    }

    #[test]
    fn increments_invariant_checks_even_when_entered_playing_false() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame(5))
            .insert_resource(ScenarioStats {
                entered_playing: false,
                invariant_checks: 0,
                ..Default::default()
            })
            .add_systems(FixedUpdate, check_breaker_count_reasonable);
        tick(&mut app);
        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.invariant_checks, 1,
            "invariant_checks should increment even when entered_playing is false, got {}",
            stats.invariant_checks
        );
    }

    // -- Behavior 15: ExtraBreaker entities are not counted by the checker --

    #[test]
    fn extra_breaker_entities_not_counted() {
        let mut app = test_app();
        app.world_mut().spawn(PrimaryBreaker);
        app.world_mut().spawn(ExtraBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when 1 PrimaryBreaker + 1 ExtraBreaker exist, got {}",
            log.0.len()
        );
    }

    #[test]
    fn extra_breaker_entities_do_not_satisfy_primary_breaker_query() {
        let mut app = test_app();
        // 0 PrimaryBreaker + 2 ExtraBreaker = count is 0 (not 2)
        app.world_mut().spawn(ExtraBreaker);
        app.world_mut().spawn(ExtraBreaker);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when 0 PrimaryBreaker + 2 ExtraBreaker, got {}",
            log.0.len()
        );
        assert!(
            log.0[0].message.contains("count=0"),
            "violation message should contain 'count=0' (ExtraBreaker not counted), got: {}",
            log.0[0].message
        );
    }
}
