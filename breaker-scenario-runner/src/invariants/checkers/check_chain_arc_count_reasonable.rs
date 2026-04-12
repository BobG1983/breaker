use bevy::prelude::*;
use breaker::effect_v3::effects::chain_lightning::{ChainLightningArc, ChainLightningChain};

use crate::{invariants::*, lifecycle::ScenarioConfig, types::InvariantKind};

/// Checks that [`ChainLightningChain`] + [`ChainLightningArc`] entities do not
/// accumulate unboundedly.
///
/// Each chain lightning effect spawns chain and arc entities that should be
/// cleaned up as arcs complete and chains finish. If despawning is broken
/// they accumulate indefinitely.
///
/// Fires when combined count exceeds `invariant_params.max_chain_arc_count` (default 50).
pub fn check_chain_arc_count_reasonable(
    chains: Query<Entity, With<ChainLightningChain>>,
    arcs: Query<Entity, With<ChainLightningArc>>,
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let max = config.definition.invariant_params.max_chain_arc_count;
    let count = chains.iter().count() + arcs.iter().count();
    if count > max {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::ChainArcCountReasonable,
            entity: None,
            message: format!(
                "ChainArcCountReasonable FAIL frame={} count={count} max={max}",
                frame.0,
            ),
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use breaker::{effect_v3::effects::chain_lightning::ChainState, state::types::NodeState};
    use rantzsoft_stateflow::CleanupOnExit;

    use super::*;
    use crate::types::{InputStrategy, InvariantParams, ScenarioDefinition, ScriptedParams};

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app(max_chain_arc_count: usize) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(ScenarioConfig {
                definition: ScenarioDefinition {
                    breaker: "Aegis".to_owned(),
                    layout: "Corridor".to_owned(),
                    input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
                    max_frames: 1000,
                    disallowed_failures: vec![],
                    invariant_params: InvariantParams {
                        max_chain_arc_count,
                        ..InvariantParams::default()
                    },
                    ..Default::default()
                },
            })
            .add_systems(FixedUpdate, check_chain_arc_count_reasonable);
        app
    }

    fn spawn_chain(world: &mut World) -> Entity {
        world
            .spawn((
                ChainLightningChain {
                    source_pos: Vec2::ZERO,
                    remaining_jumps: 0,
                    damage: 0.0,
                    hit_set: HashSet::new(),
                    state: ChainState::Idle,
                    range: 0.0,
                    arc_speed: 0.0,
                },
                CleanupOnExit::<NodeState>::default(),
            ))
            .id()
    }

    fn spawn_arc(world: &mut World) -> Entity {
        world
            .spawn((ChainLightningArc, CleanupOnExit::<NodeState>::default()))
            .id()
    }

    // -----------------------------------------------------------------
    // Behavior 9: No violation when no chain/arc entities exist
    // -----------------------------------------------------------------

    #[test]
    fn no_violation_when_no_chain_or_arc_entities_exist() {
        let mut app = test_app(50);
        // Spawn an unrelated entity to verify no false positive
        app.world_mut().spawn(Transform::default());
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no ChainLightningChain or ChainLightningArc entities exist"
        );
    }

    // -----------------------------------------------------------------
    // Behavior 10: No violation when combined count equals max
    // -----------------------------------------------------------------

    #[test]
    fn no_violation_when_combined_count_equals_max() {
        let mut app = test_app(5);
        // 3 chains + 2 arcs = 5 total = max
        for _ in 0..3 {
            spawn_chain(app.world_mut());
        }
        for _ in 0..2 {
            spawn_arc(app.world_mut());
        }
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 10;
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when chain+arc count equals max (count=5, max=5)"
        );
    }

    #[test]
    fn no_violation_when_all_chains_no_arcs_at_max() {
        let mut app = test_app(5);
        // 5 chains + 0 arcs = 5 total = max
        for _ in 0..5 {
            spawn_chain(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when 5 chains + 0 arcs = max of 5"
        );
    }

    // -----------------------------------------------------------------
    // Behavior 11: Fires when combined count exceeds max
    // -----------------------------------------------------------------

    #[test]
    fn fires_when_combined_count_exceeds_max() {
        let mut app = test_app(5);
        // 3 chains + 3 arcs = 6 > max of 5
        for _ in 0..3 {
            spawn_chain(app.world_mut());
        }
        for _ in 0..3 {
            spawn_arc(app.world_mut());
        }
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 10;
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one ChainArcCountReasonable violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ChainArcCountReasonable);
    }

    #[test]
    fn fires_when_only_arcs_exceed_max() {
        let mut app = test_app(5);
        // 0 chains + 6 arcs = 6 > max of 5
        for _ in 0..6 {
            spawn_arc(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected violation when 0 chains + 6 arcs exceeds max of 5"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ChainArcCountReasonable);
    }

    #[test]
    fn fires_when_only_chains_exceed_max() {
        let mut app = test_app(5);
        // 6 chains + 0 arcs = 6 > max of 5
        for _ in 0..6 {
            spawn_chain(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected violation when 6 chains + 0 arcs exceeds max of 5"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ChainArcCountReasonable);
    }

    // -----------------------------------------------------------------
    // Behavior 12: Violation message includes count and max values
    // -----------------------------------------------------------------

    #[test]
    fn violation_message_includes_count_and_max() {
        let mut app = test_app(5);
        // 8 chains + 4 arcs = 12 > max of 5
        for _ in 0..8 {
            spawn_chain(app.world_mut());
        }
        for _ in 0..4 {
            spawn_arc(app.world_mut());
        }
        app.world_mut().resource_mut::<ScenarioFrame>().0 = 42;
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0[0].message.contains("count=12"),
            "violation message should include count=12, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("max=5"),
            "violation message should include max=5, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("frame=42"),
            "violation message should include frame=42, got: {}",
            log.0[0].message
        );
    }

    // -----------------------------------------------------------------
    // Behavior 13: Uses scenario params for max threshold
    // -----------------------------------------------------------------

    #[test]
    fn uses_scenario_params_for_max() {
        // max=50 (default) — 25 entities should be OK
        let mut app = test_app(50);
        for _ in 0..15 {
            spawn_chain(app.world_mut());
        }
        for _ in 0..10 {
            spawn_arc(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "25 chain+arc entities should be OK with max_chain_arc_count=50"
        );
    }

    #[test]
    fn uses_scenario_params_fires_with_lower_max() {
        // max=24 — same 25 entities should fire
        let mut app = test_app(24);
        for _ in 0..15 {
            spawn_chain(app.world_mut());
        }
        for _ in 0..10 {
            spawn_arc(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "25 chain+arc entities should fire with max_chain_arc_count=24"
        );
    }

    // -----------------------------------------------------------------
    // Behavior 14: Chains and arcs independently contribute to total
    // -----------------------------------------------------------------

    #[test]
    fn chains_alone_exceed_threshold() {
        let mut app = test_app(3);
        // 4 chains + 0 arcs = 4 > max of 3
        for _ in 0..4 {
            spawn_chain(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1, "4 chains alone should exceed max of 3");
    }

    #[test]
    fn arcs_alone_exceed_threshold() {
        let mut app = test_app(3);
        // 0 chains + 4 arcs = 4 > max of 3
        for _ in 0..4 {
            spawn_arc(app.world_mut());
        }
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1, "4 arcs alone should exceed max of 3");
    }
}
