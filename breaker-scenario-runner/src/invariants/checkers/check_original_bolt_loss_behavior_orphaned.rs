use bevy::prelude::*;
use breaker::mutators::protocols::{
    definition::ProtocolKind, reckless_dash::OriginalBoltLossBehavior, resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: `OriginalBoltLossBehavior` components on entities are only
/// legal while `ActiveProtocols` contains `ProtocolKind::RecklessDash`.
///
/// Violation indicates orphaned reckless-dash overlay state — either:
/// - `ActiveProtocols` was cleared without removing the overlay component, or
/// - a cross-node state leak carried the overlay from a prior node, causing
///   permanent doubling of `BoltLossBehavior` for the remainder of the run.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_original_bolt_loss_behavior_orphaned(
    overlay_query: Query<Entity, With<OriginalBoltLossBehavior>>,
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

    let reckless_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::RecklessDash));

    if reckless_active {
        return;
    }

    // RecklessDash not active — check for orphaned overlay state.
    for entity in overlay_query.iter() {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::OriginalBoltLossBehaviorOrphaned,
            entity:    Some(entity),
            message:   format!(
                "OriginalBoltLossBehaviorOrphaned FAIL frame={} entity={entity:?} has OriginalBoltLossBehavior but RecklessDash is not active",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::{
        breaker::components::BoltLossBehavior,
        mutators::protocols::{
            definition::{ProtocolDefinition, ProtocolTuning},
            reckless_dash::OriginalBoltLossBehavior,
            resources::ActiveProtocols,
        },
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
            .add_systems(FixedUpdate, check_original_bolt_loss_behavior_orphaned);
        app
    }

    fn reckless_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "RecklessDash".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::RecklessDash {
                risky_zone_start:  0.5,
                damage_multiplier: 2.0,
                double_penalty:    true,
            },
        }
    }

    // -- no OriginalBoltLossBehavior components → no violation --

    #[test]
    fn no_overlay_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when no overlay components, got {}",
            log.0.len()
        );
    }

    // -- OriginalBoltLossBehavior present, RecklessDash active → no violation --

    #[test]
    fn overlay_present_reckless_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(reckless_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut()
            .spawn(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when overlay is present and RecklessDash is active, got {}",
            log.0.len()
        );
    }

    // -- OriginalBoltLossBehavior present, RecklessDash not active → VIOLATION per entity --

    #[test]
    fn overlay_present_reckless_inactive_fires_violation() {
        let mut app = test_app();
        app.world_mut()
            .spawn(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for 1 orphaned overlay, got {}",
            log.0.len()
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::OriginalBoltLossBehaviorOrphaned
        );
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
    }

    // -- multiple overlay components → one violation per entity --

    #[test]
    fn multiple_overlays_fires_one_violation_per_entity() {
        let mut app = test_app();
        app.world_mut()
            .spawn(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));
        app.world_mut()
            .spawn(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(2)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for 2 orphaned overlays, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::OriginalBoltLossBehaviorOrphaned),
            "all violations should be OriginalBoltLossBehaviorOrphaned"
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
            .add_systems(FixedUpdate, check_original_bolt_loss_behavior_orphaned);
        app.world_mut()
            .spawn(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));
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
        app.world_mut()
            .spawn(OriginalBoltLossBehavior(BoltLossBehavior::LifeLoss(1)));
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
}
