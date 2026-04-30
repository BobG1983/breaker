use bevy::prelude::*;
use breaker::mutators::protocols::{
    definition::ProtocolKind, reckless_dash::RiskyDamageBoost, resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: `RiskyDamageBoost` components on bolts are only legal
/// while `ActiveProtocols` contains `ProtocolKind::RecklessDash`.
///
/// Violation indicates orphaned reckless-dash state — either:
/// - `ActiveProtocols` was cleared without removing reckless-dash components, or
/// - a cross-node state leak carried boost markers from a prior node.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_reckless_dash_orphaned(
    boost_query: Query<Entity, With<RiskyDamageBoost>>,
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

    // RecklessDash not active — check for orphaned component state.
    for entity in boost_query.iter() {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::RecklessDashOrphaned,
            entity:    Some(entity),
            message:   format!(
                "RecklessDashOrphaned FAIL frame={} entity={entity:?} has RiskyDamageBoost but RecklessDash is not active",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        reckless_dash::RiskyDamageBoost,
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
            .add_systems(FixedUpdate, check_reckless_dash_orphaned);
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

    // -- no RiskyDamageBoost components → no violation --

    #[test]
    fn no_boost_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when no boost components, got {}",
            log.0.len()
        );
    }

    // -- RiskyDamageBoost present, RecklessDash active → no violation --

    #[test]
    fn boost_present_reckless_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(reckless_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut().spawn(RiskyDamageBoost { multiplier: 2.0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when boost is present and RecklessDash is active, got {}",
            log.0.len()
        );
    }

    // -- RiskyDamageBoost present, RecklessDash not active → VIOLATION per entity --

    #[test]
    fn boost_present_reckless_inactive_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(RiskyDamageBoost { multiplier: 1.5 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for 1 orphaned boost, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::RecklessDashOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
    }

    // -- multiple boost components → one violation per entity --

    #[test]
    fn multiple_boosts_fires_one_violation_per_entity() {
        let mut app = test_app();
        app.world_mut().spawn(RiskyDamageBoost { multiplier: 2.0 });
        app.world_mut().spawn(RiskyDamageBoost { multiplier: 3.0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for 2 orphaned boosts, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::RecklessDashOrphaned),
            "all violations should be RecklessDashOrphaned"
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
            .add_systems(FixedUpdate, check_reckless_dash_orphaned);
        app.world_mut().spawn(RiskyDamageBoost { multiplier: 2.0 });
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
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 55;
        app.world_mut().spawn(RiskyDamageBoost { multiplier: 1.0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=55"),
            "message should contain frame=55, got: {}",
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
