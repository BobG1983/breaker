use bevy::prelude::*;
use breaker::mutators::protocols::{
    definition::ProtocolKind, greed::GreedStacks, resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: `GreedStacks.skips > 0` is only legal while
/// `ActiveProtocols` contains `ProtocolKind::Greed`.
///
/// Violation indicates orphaned skip state — either:
/// - `reset_run_state` failed to clear `GreedStacks` at run boundary, or
/// - `ActiveProtocols` was cleared without clearing `GreedStacks`, or
/// - a cross-node state leak carried skips from a prior run.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_greed_stacks_orphaned(
    greed_stacks: Option<Res<GreedStacks>>,
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

    let Some(stacks) = greed_stacks else {
        return;
    };

    if stacks.skips == 0 {
        return;
    }

    // skips > 0 — Greed MUST be in ActiveProtocols
    let greed_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::Greed));

    if !greed_active {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::GreedStacksOrphaned,
            entity:    None,
            message:   format!(
                "GreedStacksOrphaned FAIL frame={} skips={} but Greed is not in ActiveProtocols",
                frame.0, stacks.skips,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        greed::GreedStacks,
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
            .add_systems(FixedUpdate, check_greed_stacks_orphaned);
        app
    }

    fn greed_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "Greed".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        }
    }

    // -- no GreedStacks resource → no violation --

    #[test]
    fn no_greed_stacks_resource_no_violation() {
        let mut app = test_app();
        // No GreedStacks inserted
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when GreedStacks absent, got {}",
            log.0.len()
        );
    }

    // -- skips = 0, Greed absent → no violation (zero is always safe) --

    #[test]
    fn skips_zero_greed_absent_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when skips=0 and Greed inactive, got {}",
            log.0.len()
        );
    }

    // -- skips = 0, Greed active → no violation --

    #[test]
    fn skips_zero_greed_active_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 0 });
        let mut protocols = ActiveProtocols::default();
        protocols.insert(greed_definition());
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when skips=0 and Greed active, got {}",
            log.0.len()
        );
    }

    // -- skips > 0, Greed active → no violation (legal state) --

    #[test]
    fn skips_nonzero_greed_active_no_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 5 });
        let mut protocols = ActiveProtocols::default();
        protocols.insert(greed_definition());
        app.world_mut().insert_resource(protocols);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when skips=5 and Greed active, got {}",
            log.0.len()
        );
    }

    // -- skips > 0, Greed absent → VIOLATION --

    #[test]
    fn skips_nonzero_greed_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 3 });
        // No ActiveProtocols inserted
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when skips=3 and Greed inactive, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::GreedStacksOrphaned);
        assert!(
            log.0[0].message.contains("skips=3"),
            "message should contain skips value, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].entity.is_none(),
            "entity should be None for resource-level checker"
        );
    }

    // -- skips > 0, ActiveProtocols present but Greed not in it → VIOLATION --

    #[test]
    fn skips_nonzero_greed_not_in_active_protocols_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 1 });
        // Insert empty ActiveProtocols (Greed not present)
        app.world_mut().insert_resource(ActiveProtocols::default());
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation when skips=1 and Greed not in ActiveProtocols, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::GreedStacksOrphaned);
    }

    // -- large skip count fires violation with correct value --

    #[test]
    fn large_skip_count_fires_violation_with_correct_value() {
        let mut app = test_app();
        app.world_mut()
            .insert_resource(GreedStacks { skips: u32::MAX });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains(&format!("skips={}", u32::MAX)),
            "message should contain the exact skip count"
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
            .add_systems(FixedUpdate, check_greed_stacks_orphaned);
        app.world_mut().insert_resource(GreedStacks { skips: 10 });
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
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 42;
        app.world_mut().insert_resource(GreedStacks { skips: 7 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=42"),
            "message should contain frame=42, got: {}",
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

    // -- ProtocolKind::Greed present but another protocol in set doesn't suppress --

    #[test]
    fn skips_nonzero_only_other_protocol_active_fires_violation() {
        let mut app = test_app();
        app.world_mut().insert_resource(GreedStacks { skips: 2 });
        // Insert active protocols with Deadline but NOT Greed
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
            "expected violation when only Deadline is active, not Greed, got {}",
            log.0.len()
        );
    }
}
