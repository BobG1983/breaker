use bevy::prelude::*;
use breaker::mutators::protocols::{
    definition::ProtocolKind, resources::ActiveProtocols, siphon::SiphonStreak,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: `SiphonStreak.kill_count > 0` is only legal while
/// `ActiveProtocols` contains `ProtocolKind::Siphon`.
///
/// Violation indicates orphaned streak state — either:
/// - `siphon_cleanup_node` failed to clear `SiphonStreak` at node boundary, or
/// - `reset_run_state` failed to clear `SiphonStreak` at run boundary, or
/// - `ActiveProtocols` was cleared without clearing `SiphonStreak`, or
/// - a cross-node state leak carried `kill_count` from a prior node.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_siphon_streak_orphaned(
    siphon_streak: Option<Res<SiphonStreak>>,
    active_protocols: Option<Res<ActiveProtocols>>,
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

    let Some(streak) = siphon_streak else {
        return;
    };

    if streak.kill_count == 0 {
        return;
    }

    // kill_count > 0 — Siphon MUST be in ActiveProtocols
    let siphon_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::Siphon));

    if !siphon_active {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::SiphonStreakOrphaned,
            entity:    None,
            message:   format!(
                "SiphonStreakOrphaned FAIL frame={} kill_count={} but Siphon is not in ActiveProtocols",
                frame.0, streak.kill_count,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
        siphon::SiphonStreak,
    };

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
            .add_systems(FixedUpdate, check_siphon_streak_orphaned);
        app
    }

    fn siphon_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "Siphon".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Siphon {
                streak_window: 3.0,
                time_per_kill: 1.5,
            },
        }
    }

    // -- no SiphonStreak resource → no violation --

    #[test]
    fn no_siphon_streak_resource_no_violation() {
        let mut app = test_app();
        // No SiphonStreak inserted
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when SiphonStreak absent, got {}",
            log.0.len()
        );
    }

    // -- kill_count = 0, Siphon absent → no violation (zero is always safe) --

    #[test]
    fn kill_count_zero_siphon_absent_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 0.0,
            kill_count:       0,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when kill_count=0 and Siphon inactive, got {}",
            log.0.len()
        );
    }

    // -- kill_count = 0, Siphon active → no violation --

    #[test]
    fn kill_count_zero_siphon_active_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 0.0,
            kill_count:       0,
        });
        let mut protocols = ActiveProtocols::default();
        protocols.insert(siphon_definition());
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when kill_count=0 and Siphon active, got {}",
            log.0.len()
        );
    }

    // -- kill_count > 0, Siphon active → no violation (legal state) --

    #[test]
    fn kill_count_nonzero_siphon_active_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 2.5,
            kill_count:       4,
        });
        let mut protocols = ActiveProtocols::default();
        protocols.insert(siphon_definition());
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when kill_count=4 and Siphon active, got {}",
            log.0.len()
        );
    }

    // -- kill_count > 0, Siphon absent → VIOLATION --

    #[test]
    fn kill_count_nonzero_siphon_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 1.0,
            kill_count:       3,
        });
        // No ActiveProtocols inserted
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when kill_count=3 and Siphon inactive, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::SiphonStreakOrphaned);
        assert!(
            log.0[0].message.contains("kill_count=3"),
            "message should contain kill_count value, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].entity.is_none(),
            "entity should be None for resource-level checker"
        );
    }

    // -- kill_count > 0, ActiveProtocols present but Siphon not in it → VIOLATION --

    #[test]
    fn kill_count_nonzero_siphon_not_in_active_protocols_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 0.5,
            kill_count:       1,
        });
        // Insert empty ActiveProtocols (Siphon not present)
        app.world_mut().insert_resource(ActiveProtocols::default());
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when kill_count=1 and Siphon not in ActiveProtocols, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::SiphonStreakOrphaned);
    }

    // -- large kill_count fires violation with correct value --

    #[test]
    fn large_kill_count_fires_violation_with_correct_value() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 0.0,
            kill_count:       u32::MAX,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0]
                .message
                .contains(&format!("kill_count={}", u32::MAX)),
            "message should contain the exact kill_count, got: {}",
            log.0[0].message
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
            .add_systems(FixedUpdate, check_siphon_streak_orphaned);
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 1.0,
            kill_count:       10,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations when entered_playing is false, got {}",
            log.0.len()
        );
    }

    // -- frame number recorded correctly --

    #[test]
    fn violation_message_contains_frame_number() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 99;
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 0.5,
            kill_count:       7,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=99"),
            "message should contain frame=99, got: {}",
            log.0[0].message
        );
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

    // -- only another protocol active but not Siphon → fires violation --

    #[test]
    fn kill_count_nonzero_only_other_protocol_active_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 1.0,
            kill_count:       2,
        });
        // Insert active protocols with Deadline but NOT Siphon
        let mut protocols = ActiveProtocols::default();
        protocols.insert(ProtocolDefinition {
            name:        "Deadline".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Deadline { effects: vec![] },
        });
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected violation when only Deadline is active, not Siphon, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::SiphonStreakOrphaned);
    }

    // -- window_remaining nonzero with kill_count zero → no violation --

    #[test]
    fn window_remaining_nonzero_kill_count_zero_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(SiphonStreak {
            window_remaining: 2.0,
            kill_count:       0,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "window_remaining nonzero but kill_count=0 should not fire, got {}",
            log.0.len()
        );
    }
}
