use bevy::prelude::*;
use breaker::protocol::{
    definition::ProtocolKind,
    protocols::burnout::{BurnoutDamageBoost, BurnoutHeat},
    resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: no breaker carries `BurnoutHeat` with `heat > 0.0` and
/// no bolt carries `BurnoutDamageBoost` when `ActiveProtocols` does not contain
/// `ProtocolKind::Burnout`.
///
/// Violation indicates orphaned burnout state — either:
/// - `burnout_cleanup_node` failed to remove burnout components at node
///   boundary, or
/// - `ActiveProtocols` was cleared without removing burnout components, or
/// - a cross-node state leak carried heat or damage boost from a prior node.
///
/// `BurnoutHeat` with `heat == 0.0` is permitted without Burnout active (the
/// component has a zero `Default`) — only positive heat values indicate
/// meaningful orphaned state.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_burnout_state_orphaned(
    heat_query: Query<(Entity, &BurnoutHeat)>,
    boost_query: Query<Entity, With<BurnoutDamageBoost>>,
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

    let burnout_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::Burnout));

    if burnout_active {
        return;
    }

    // Burnout not active — check for orphaned component state.
    for (entity, heat) in heat_query.iter() {
        if heat.heat > 0.0 {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::BurnoutStateOrphaned,
                entity:    Some(entity),
                message:   format!(
                    "BurnoutStateOrphaned FAIL frame={} entity={entity:?} has BurnoutHeat.heat={} but Burnout is not active",
                    frame.0, heat.heat,
                ),
            });
        }
    }

    for entity in boost_query.iter() {
        log.0.push(ViolationEntry {
            frame:     frame.0,
            invariant: InvariantKind::BurnoutStateOrphaned,
            entity:    Some(entity),
            message:   format!(
                "BurnoutStateOrphaned FAIL frame={} entity={entity:?} has BurnoutDamageBoost but Burnout is not active",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use breaker::protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        protocols::burnout::{BurnoutDamageBoost, BurnoutHeat},
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
            .add_systems(FixedUpdate, check_burnout_state_orphaned);
        app
    }

    fn burnout_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "Burnout".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Burnout {
                fill_duration:               5.0,
                drain_duration:              3.0,
                still_threshold:             1.0,
                full_heat_damage_multiplier: 3.0,
                speed_boost_duration:        0.5,
            },
        }
    }

    // -- no burnout components → no violation --

    #[test]
    fn no_burnout_components_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when no burnout components present, got {}",
            log.0.len()
        );
    }

    // -- BurnoutHeat.heat == 0.0, Burnout absent → no violation (zero is legal) --

    #[test]
    fn zero_heat_burnout_absent_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.0,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for BurnoutHeat.heat=0.0 when Burnout absent, got {}",
            log.0.len()
        );
    }

    // -- BurnoutHeat.heat > 0.0, Burnout active → no violation --

    #[test]
    fn positive_heat_burnout_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(burnout_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.5,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when heat positive and Burnout active, got {}",
            log.0.len()
        );
    }

    // -- BurnoutHeat.heat > 0.0, Burnout absent → VIOLATION --

    #[test]
    fn positive_heat_burnout_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.75,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for orphaned positive BurnoutHeat, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutStateOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
        assert!(
            log.0[0].message.contains("BurnoutHeat"),
            "message should mention BurnoutHeat, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("0.75"),
            "message should include the heat value, got: {}",
            log.0[0].message
        );
    }

    // -- BurnoutDamageBoost present, Burnout active → no violation --

    #[test]
    fn damage_boost_burnout_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(burnout_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut()
            .spawn(BurnoutDamageBoost { multiplier: 3.0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when BurnoutDamageBoost present and Burnout active, got {}",
            log.0.len()
        );
    }

    // -- BurnoutDamageBoost present, Burnout absent → VIOLATION --

    #[test]
    fn damage_boost_burnout_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut()
            .spawn(BurnoutDamageBoost { multiplier: 2.0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for orphaned BurnoutDamageBoost, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutStateOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
        assert!(
            log.0[0].message.contains("BurnoutDamageBoost"),
            "message should mention BurnoutDamageBoost, got: {}",
            log.0[0].message
        );
    }

    // -- entity with positive heat + separate entity with damage boost → 2 violations --

    #[test]
    fn heat_and_boost_on_separate_entities_each_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.3,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        app.world_mut()
            .spawn(BurnoutDamageBoost { multiplier: 1.5 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for heat entity + boost entity, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::BurnoutStateOrphaned),
            "all violations should be BurnoutStateOrphaned"
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
            .add_systems(FixedUpdate, check_burnout_state_orphaned);
        app.world_mut().spawn(BurnoutHeat {
            heat:              1.0,
            still_timer:       0.0,
            mega_bump_charged: true,
        });
        app.world_mut()
            .spawn(BurnoutDamageBoost { multiplier: 3.0 });
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
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 33;
        app.world_mut().spawn(BurnoutHeat {
            heat:              0.1,
            still_timer:       0.0,
            mega_bump_charged: false,
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=33"),
            "message should contain frame=33, got: {}",
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

    // -- only another protocol active but not Burnout → fires violation --

    #[test]
    fn burnout_component_present_only_other_protocol_active_fires_violation() {
        let mut app = test_app();
        app.world_mut()
            .spawn(BurnoutDamageBoost { multiplier: 2.0 });
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
            "expected violation when only Deadline is active, not Burnout, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BurnoutStateOrphaned);
    }
}
