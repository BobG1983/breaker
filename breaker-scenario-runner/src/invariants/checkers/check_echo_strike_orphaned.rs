use bevy::prelude::*;
use breaker::mutators::protocols::{
    definition::ProtocolKind,
    echo_strike::{EchoNetwork, EchoPrimed},
    resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: no bolt carries `EchoNetwork` or `EchoPrimed` when
/// `ActiveProtocols` does not contain `ProtocolKind::EchoStrike`.
///
/// Violation indicates orphaned echo state — either:
/// - `echo_strike_cleanup_node` failed to remove echo components at node
///   boundary, or
/// - `ActiveProtocols` was cleared without removing echo components, or
/// - a cross-node state leak carried echo markers from a prior node.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_echo_strike_orphaned(
    echo_query: Query<(Entity, Has<EchoNetwork>, Has<EchoPrimed>)>,
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

    let echo_strike_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::EchoStrike));

    if echo_strike_active {
        return;
    }

    // EchoStrike not active — check for orphaned component state.
    for (entity, has_network, has_primed) in echo_query.iter() {
        if has_network {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::EchoStrikeOrphaned,
                entity:    Some(entity),
                message:   format!(
                    "EchoStrikeOrphaned FAIL frame={} entity={entity:?} has EchoNetwork but EchoStrike is not active",
                    frame.0,
                ),
            });
        }
        if has_primed {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::EchoStrikeOrphaned,
                entity:    Some(entity),
                message:   format!(
                    "EchoStrikeOrphaned FAIL frame={} entity={entity:?} has EchoPrimed but EchoStrike is not active",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        echo_strike::{EchoNetwork, EchoPrimed},
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
            .add_systems(FixedUpdate, check_echo_strike_orphaned);
        app
    }

    fn echo_strike_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "EchoStrike".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::EchoStrike {
                max_echoes:      3,
                newest_fraction: 0.5,
                middle_fraction: 0.3,
                oldest_fraction: 0.2,
            },
        }
    }

    // -- no echo components, EchoStrike absent → no violation --

    #[test]
    fn no_echo_components_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when no echo components present, got {}",
            log.0.len()
        );
    }

    // -- EchoNetwork present, EchoStrike active → no violation --

    #[test]
    fn echo_network_present_echo_strike_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(echo_strike_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut().spawn(EchoNetwork {
            echoes: std::collections::VecDeque::new(),
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when EchoNetwork present and EchoStrike active, got {}",
            log.0.len()
        );
    }

    // -- EchoPrimed present, EchoStrike active → no violation --

    #[test]
    fn echo_primed_present_echo_strike_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(echo_strike_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut().spawn(EchoPrimed);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when EchoPrimed present and EchoStrike active, got {}",
            log.0.len()
        );
    }

    // -- EchoNetwork present, EchoStrike not active → VIOLATION --

    #[test]
    fn echo_network_present_echo_strike_inactive_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(EchoNetwork {
            echoes: std::collections::VecDeque::new(),
        });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for orphaned EchoNetwork, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::EchoStrikeOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
        assert!(
            log.0[0].message.contains("EchoNetwork"),
            "message should mention EchoNetwork, got: {}",
            log.0[0].message
        );
    }

    // -- EchoPrimed present, EchoStrike not active → VIOLATION --

    #[test]
    fn echo_primed_present_echo_strike_inactive_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(EchoPrimed);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for orphaned EchoPrimed, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::EchoStrikeOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
        assert!(
            log.0[0].message.contains("EchoPrimed"),
            "message should mention EchoPrimed, got: {}",
            log.0[0].message
        );
    }

    // -- entity with both EchoNetwork and EchoPrimed → 2 violations --

    #[test]
    fn entity_with_both_components_fires_two_violations() {
        let mut app = test_app();
        app.world_mut().spawn((
            EchoNetwork {
                echoes: std::collections::VecDeque::new(),
            },
            EchoPrimed,
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for entity with both echo components, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::EchoStrikeOrphaned),
            "all violations should be EchoStrikeOrphaned"
        );
    }

    // -- multiple entities each fire their own violations --

    #[test]
    fn multiple_entities_fire_one_violation_each() {
        let mut app = test_app();
        app.world_mut().spawn(EchoNetwork {
            echoes: std::collections::VecDeque::new(),
        });
        app.world_mut().spawn(EchoPrimed);
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for 2 entities with echo components, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::EchoStrikeOrphaned),
            "all violations should be EchoStrikeOrphaned"
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
            .add_systems(FixedUpdate, check_echo_strike_orphaned);
        app.world_mut().spawn(EchoNetwork {
            echoes: std::collections::VecDeque::new(),
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
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 42;
        app.world_mut().spawn(EchoPrimed);
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

    // -- only another protocol active but not EchoStrike → fires violation --

    #[test]
    fn echo_component_present_only_other_protocol_active_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(EchoPrimed);
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
            "expected violation when only Deadline is active, not EchoStrike, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::EchoStrikeOrphaned);
    }
}
