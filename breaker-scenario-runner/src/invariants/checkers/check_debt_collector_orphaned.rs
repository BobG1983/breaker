use bevy::prelude::*;
use breaker::protocol::{
    definition::ProtocolKind,
    protocols::debt_collector::{DebtCashOut, DebtStack},
    resources::ActiveProtocols,
};

use crate::{invariants::*, types::InvariantKind};

/// CONTRACT invariant: no bolt carries `DebtStack` (value > 0.0) or
/// `DebtCashOut` when `ActiveProtocols` does not contain
/// `ProtocolKind::DebtCollector`.
///
/// Violation indicates orphaned debt state — either:
/// - `debt_collector_cleanup_node` failed to remove debt components at node
///   boundary, or
/// - `ActiveProtocols` was cleared without removing debt components, or
/// - a cross-node state leak carried debt markers from a prior node.
///
/// `DebtStack` with value 0.0 is permitted without `DebtCollector` active —
/// only positive stack values indicate meaningful orphaned state.
///
/// Gated on [`ScenarioStats::entered_playing`]: returns without producing
/// violations when the game has not yet entered the `Playing` state.
///
/// Increments [`ScenarioStats::invariant_checks`] when it runs.
pub fn check_debt_collector_orphaned(
    debt_query: Query<(Entity, &DebtStack, Has<DebtCashOut>)>,
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

    let debt_collector_active = active_protocols
        .as_ref()
        .is_some_and(|ap| ap.contains(ProtocolKind::DebtCollector));

    if debt_collector_active {
        return;
    }

    // DebtCollector not active — check for orphaned component state.
    for (entity, stack, has_cash_out) in debt_query.iter() {
        if stack.0 > 0.0 {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::DebtCollectorOrphaned,
                entity:    Some(entity),
                message:   format!(
                    "DebtCollectorOrphaned FAIL frame={} entity={entity:?} has DebtStack({}) but DebtCollector is not active",
                    frame.0, stack.0,
                ),
            });
        }
        if has_cash_out {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::DebtCollectorOrphaned,
                entity:    Some(entity),
                message:   format!(
                    "DebtCollectorOrphaned FAIL frame={} entity={entity:?} has DebtCashOut but DebtCollector is not active",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use breaker::protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        protocols::debt_collector::{DebtCashOut, DebtStack},
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
            .add_systems(FixedUpdate, check_debt_collector_orphaned);
        app
    }

    fn debt_collector_definition() -> ProtocolDefinition {
        ProtocolDefinition {
            name:        "DebtCollector".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::DebtCollector {
                stack_per_bump: 0.1,
            },
        }
    }

    // -- no debt components → no violation --

    #[test]
    fn no_debt_components_no_violation() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when no debt components present, got {}",
            log.0.len()
        );
    }

    // -- DebtStack(0.0), DebtCollector absent → no violation (zero is legal) --

    #[test]
    fn zero_debt_stack_debt_collector_absent_no_violation() {
        let mut app = test_app();
        app.world_mut().spawn(DebtStack(0.0));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for DebtStack(0.0) when DebtCollector absent, got {}",
            log.0.len()
        );
    }

    // -- DebtStack(positive), DebtCollector active → no violation --

    #[test]
    fn positive_debt_stack_debt_collector_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(debt_collector_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut().spawn(DebtStack(2.5));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when DebtStack positive and DebtCollector active, got {}",
            log.0.len()
        );
    }

    // -- DebtStack(positive), DebtCollector absent → VIOLATION --

    #[test]
    fn positive_debt_stack_debt_collector_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(DebtStack(1.5));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for orphaned DebtStack, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::DebtCollectorOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
        assert!(
            log.0[0].message.contains("DebtStack"),
            "message should mention DebtStack, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("1.5"),
            "message should include the stack value, got: {}",
            log.0[0].message
        );
    }

    // -- DebtCashOut present, DebtCollector active → no violation --

    #[test]
    fn debt_cash_out_debt_collector_active_no_violation() {
        let mut app = test_app();
        let mut protocols = ActiveProtocols::default();
        protocols.insert(debt_collector_definition());
        app.world_mut().insert_resource(protocols);
        app.world_mut().spawn((DebtStack(0.0), DebtCashOut(3.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when DebtCashOut present and DebtCollector active, got {}",
            log.0.len()
        );
    }

    // -- DebtCashOut present, DebtCollector absent → VIOLATION --

    #[test]
    fn debt_cash_out_debt_collector_absent_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn((DebtStack(0.0), DebtCashOut(2.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 violation for orphaned DebtCashOut, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::DebtCollectorOrphaned);
        assert!(
            log.0[0].entity.is_some(),
            "entity field should be set for component-level violation"
        );
        assert!(
            log.0[0].message.contains("DebtCashOut"),
            "message should mention DebtCashOut, got: {}",
            log.0[0].message
        );
    }

    // -- entity with both positive DebtStack and DebtCashOut → 2 violations --

    #[test]
    fn entity_with_both_debt_components_fires_two_violations() {
        let mut app = test_app();
        app.world_mut().spawn((DebtStack(1.0), DebtCashOut(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            2,
            "expected 2 violations for entity with positive DebtStack and DebtCashOut, got {}",
            log.0.len()
        );
        assert!(
            log.0
                .iter()
                .all(|v| v.invariant == InvariantKind::DebtCollectorOrphaned),
            "all violations should be DebtCollectorOrphaned"
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
            .add_systems(FixedUpdate, check_debt_collector_orphaned);
        app.world_mut().spawn(DebtStack(5.0));
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
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 88;
        app.world_mut().spawn(DebtStack(0.5));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert!(
            log.0[0].message.contains("frame=88"),
            "message should contain frame=88, got: {}",
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

    // -- only another protocol active but not DebtCollector → fires violation --

    #[test]
    fn debt_present_only_other_protocol_active_fires_violation() {
        let mut app = test_app();
        app.world_mut().spawn(DebtStack(1.0));
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
            "expected violation when only Deadline is active, not DebtCollector, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::DebtCollectorOrphaned);
    }
}
