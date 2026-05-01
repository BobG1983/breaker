use bevy::prelude::*;
use breaker::mutators::protocols::{
    definition::ProtocolKind, fission::FissionCounter, resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: `FissionCounter.kills > 0` is only legal while
/// `ActiveProtocols` contains `ProtocolKind::Fission`.
///
/// Violation indicates orphaned kill-count state — either:
/// - `fission_cleanup_run` failed to remove `FissionCounter` at run boundary, or
/// - `ActiveProtocols` was cleared without removing `FissionCounter`, or
/// - a cross-run state leak carried `kills` from a prior run.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_fission_counter_orphaned(
    fission_counter: Option<Res<FissionCounter>>,
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

    let Some(counter) = fission_counter else {
        return;
    };

    if counter.kills == 0 {
        return;
    }

    // kills > 0 — Fission MUST be in ActiveProtocols
    let fission_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::Fission));

    if !fission_active {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::FissionCounterOrphaned,
            entity:    None,
            message:   format!(
                "FissionCounterOrphaned FAIL frame={} kills={} but Fission is not in ActiveProtocols",
                frame.0, counter.kills,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        fission::FissionCounter,
        resources::ActiveProtocols,
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
            .add_systems(FixedUpdate, check_fission_counter_orphaned);
        app
    }

    fn fission_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "Fission".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Fission {
                kills_per_split:      5,
                divergence_angle_rad: 0.0,
            },
        }
    }

    // -- no FissionCounter resource → no violation --

    #[test]
    fn no_fission_counter_resource_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when FissionCounter absent, got {}",
            log.0.len()
        );
    }

    // -- kills = 0, Fission absent → no violation --

    #[test]
    fn kills_zero_fission_absent_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(FissionCounter { kills: 0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when kills=0 and Fission inactive, got {}",
            log.0.len()
        );
    }

    // -- kills = 0, Fission active → no violation --

    #[test]
    fn kills_zero_fission_active_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(FissionCounter { kills: 0 });
        let mut protocols = ActiveProtocols::default();
        protocols.insert(fission_definition());
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when kills=0 and Fission active, got {}",
            log.0.len()
        );
    }

    // -- kills > 0, Fission active → no violation (legal state) --

    #[test]
    fn kills_nonzero_fission_active_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(FissionCounter { kills: 3 });
        let mut protocols = ActiveProtocols::default();
        protocols.insert(fission_definition());
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when kills=3 and Fission active, got {}",
            log.0.len()
        );
    }

    // -- kills > 0, Fission absent → VIOLATION --

    #[test]
    fn kills_nonzero_fission_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(FissionCounter { kills: 2 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when kills=2 and Fission inactive, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::FissionCounterOrphaned);
        assert!(
            log.0[0].message.contains("kills=2"),
            "message should contain kills value, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].entity.is_none(),
            "entity should be None for resource-level checker"
        );
    }

    // -- kills > 0, ActiveProtocols present but Fission not in it → VIOLATION --

    #[test]
    fn kills_nonzero_fission_not_in_active_protocols_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(FissionCounter { kills: 1 });
        app.world_mut().insert_resource(ActiveProtocols::default());
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when kills=1 and Fission not in ActiveProtocols, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::FissionCounterOrphaned);
    }

    // -- large kills fires violation with correct value --

    #[test]
    fn large_kills_fires_violation_with_correct_value() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(FissionCounter { kills: u32::MAX });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains(&format!("kills={}", u32::MAX)),
            "message should contain the exact kills value, got: {}",
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
            .add_systems(FixedUpdate, check_fission_counter_orphaned);
        app.world_mut()
            .insert_resource(FissionCounter { kills: 10 });
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
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 77;
        app.world_mut().insert_resource(FissionCounter { kills: 4 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=77"),
            "message should contain frame=77, got: {}",
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

    // -- only another protocol active but not Fission → fires violation --

    #[test]
    fn kills_nonzero_only_other_protocol_active_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(FissionCounter { kills: 2 });
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
            "expected violation when only Deadline is active, not Fission, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::FissionCounterOrphaned);
    }
}
