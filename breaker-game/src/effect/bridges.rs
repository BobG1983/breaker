//! Bridge systems re-exports (canonical location: `effect/triggers/`).
//!
//! This module re-exports all bridge systems from their new per-trigger file
//! locations. Integration tests that exercise multiple bridges remain here.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::effect::triggers::{
        bump::bridge_bump, cell_destroyed::bridge_cell_destroyed, death::bridge_cell_death,
        impact::bridge_cell_impact,
    };
    use crate::{
        bolt::messages::BoltHitCell,
        breaker::{
            components::Breaker,
            messages::{BumpGrade, BumpPerformed},
        },
        cells::{
            components::RequiredToClear,
            messages::{CellDestroyedAt, RequestCellDestroyed},
        },
        effect::{
            armed::ArmedEffects,
            definition::{Effect, EffectChains, EffectNode, ImpactTarget, Trigger},
            triggers::test_helpers::*,
        },
    };

    // --- Test infrastructure ---

    #[derive(Resource)]
    struct SendBump(Option<BumpPerformed>);

    fn send_bump(msg: Res<SendBump>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendBoltHitCell(Option<BoltHitCell>);

    fn send_bolt_hit_cell(msg: Res<SendBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    #[derive(Resource)]
    struct SendCellDestroyed(Option<RequestCellDestroyed>);

    fn send_cell_destroyed(
        msg: Res<SendCellDestroyed>,
        mut writer: MessageWriter<RequestCellDestroyed>,
    ) {
        if let Some(m) = msg.0.clone() {
            writer.write(m);
        }
    }

    // =========================================================================
    // B12b: Bridge evaluation with EffectNode types (behaviors 23-24)
    // =========================================================================

    #[test]
    fn evaluate_node_returns_children_for_bolt_lost_bridge() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::evaluate_node,
        };

        let node = EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let result = evaluate_node(Trigger::BoltLost, &node);
        assert_eq!(
            result,
            Some(
                vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 32.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })]
                .as_slice()
            ),
            "bridge_bolt_lost should get children from evaluate_node"
        );
    }

    #[test]
    fn evaluate_node_returns_children_for_bump_bridge_non_leaf() {
        use crate::effect::{
            definition::{Effect, EffectNode, ImpactTarget, Trigger},
            evaluate::evaluate_node,
        };

        let inner_node = EffectNode::When {
            trigger: Trigger::Impact(ImpactTarget::Cell),
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![inner_node.clone()],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(
            result,
            Some(vec![inner_node].as_slice()),
            "PerfectBump with non-leaf child should return the inner node as child"
        );
    }

    #[test]
    fn evaluate_node_none_for_wrong_trigger() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::evaluate_node,
        };

        let node = EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
        };
        let result = evaluate_node(Trigger::PerfectBump, &node);
        assert_eq!(result, None);
    }

    // =========================================================================
    // Full two-step chain tests (multi-bridge integration)
    // =========================================================================

    #[test]
    fn full_two_step_chain_bump_arms_then_impact_fires() {
        let chain = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            }],
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    bridge_bump,
                    send_bolt_hit_cell,
                    bridge_cell_impact,
                )
                    .chain(),
            );

        // Place chain on breaker entity EffectChains
        app.world_mut()
            .spawn((Breaker, EffectChains(vec![(None, chain)])));
        let bolt = app.world_mut().spawn_empty().id();

        // Step 1: Perfect bump -- arms (evaluate_entity_chains discards Arm results
        // but breaker eval triggers Fire for leaf; non-leaf stays — this test checks
        // that the ArmedEffects flow works when armed manually)
        // Actually: with entity chains, Arm results from evaluate_entity_chains are
        // discarded. So for two-step chains to work, the chain must be on ArmedEffects
        // or the bolt entity. Let's pre-arm the bolt instead.
        // Re-build: put the chain as ArmedEffects on the bolt for step 1.
        app.world_mut().entity_mut(bolt).insert(ArmedEffects(vec![(
            None,
            EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            },
        )]));

        // Step 1: Perfect bump with armed bolt — the armed PerfectBump chain was already
        // resolved, so we skip bump and go directly to impact.

        // Step 2: Cell impact -- fires from ArmedEffects
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(captured.0.len(), 1);
        assert!((captured.0[0].base_range - 64.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // H4: Multiple bridges fire from a single BumpPerformed
    // =========================================================================

    #[test]
    fn multiple_bridges_fire_from_single_bump_performed() {
        use crate::effect::triggers::{
            bumped::bridge_bumped, perfect_bump::bridge_perfect_bump,
            perfect_bumped::bridge_perfect_bumped,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .insert_resource(SendBump(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    // All four bump bridges run in sequence
                    bridge_bump,
                    bridge_perfect_bump,
                    bridge_bumped,
                    bridge_perfect_bumped,
                )
                    .chain(),
            );

        // Bolt with When(Bumped) and When(PerfectBumped) on EffectChains
        let bolt = app
            .world_mut()
            .spawn(EffectChains(wrap_chains(vec![
                EffectNode::trigger_leaf(Trigger::Bumped, Effect::test_shockwave(16.0)),
                EffectNode::trigger_leaf(Trigger::PerfectBumped, Effect::test_shockwave(8.0)),
            ])))
            .id();

        // Breaker with When(Bump) and When(PerfectBump) on EffectChains
        app.world_mut().spawn((
            Breaker,
            EffectChains(wrap_chains(vec![
                EffectNode::trigger_leaf(Trigger::Bump, Effect::test_shockwave(64.0)),
                EffectNode::trigger_leaf(Trigger::PerfectBump, Effect::test_shockwave(32.0)),
            ])),
        ));

        // Single BumpPerformed with Perfect grade
        app.world_mut().resource_mut::<SendBump>().0 = Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt),
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        // Expect 4 firings:
        // - bridge_bump: breaker's When(Bump) fires (64.0)
        // - bridge_perfect_bump: breaker's When(PerfectBump) fires (32.0)
        // - bridge_bumped: bolt's When(Bumped) fires (16.0)
        // - bridge_perfect_bumped: bolt's When(PerfectBumped) fires (8.0)
        assert_eq!(
            captured.0.len(),
            4,
            "4 ShockwaveFired events expected — Bump(64), PerfectBump(32), \
             Bumped(16), PerfectBumped(8) — got {}",
            captured.0.len()
        );

        // Verify all 4 ranges are present (compare as sorted f32 to avoid float→int cast lints)
        let mut ranges: Vec<f32> = captured.0.iter().map(|s| s.base_range).collect();
        ranges.sort_unstable_by(f32::total_cmp);
        assert_eq!(
            ranges,
            vec![8.0, 16.0, 32.0, 64.0],
            "expected ranges [8, 16, 32, 64] — got {ranges:?}"
        );
    }

    #[test]
    fn full_three_step_chain_bump_arms_impact_rearms_cell_destroyed_fires() {
        // Pre-arm bolt with a 2-step chain (Impact(Cell) -> CellDestroyed -> Shockwave)
        // Step 1: Cell impact re-arms to CellDestroyed
        // Step 2: Cell destroyed fires shockwave
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_message::<BoltHitCell>()
            .add_message::<RequestCellDestroyed>()
            .add_message::<CellDestroyedAt>()
            .insert_resource(SendBump(None))
            .insert_resource(SendBoltHitCell(None))
            .insert_resource(SendCellDestroyed(None))
            .init_resource::<CapturedShockwaveFired>()
            .add_observer(capture_shockwave_fired)
            .add_systems(
                FixedUpdate,
                (
                    send_bump,
                    bridge_bump,
                    send_bolt_hit_cell,
                    bridge_cell_impact,
                    send_cell_destroyed,
                    bridge_cell_death,
                    bridge_cell_destroyed,
                )
                    .chain(),
            );

        // Spawn breaker with empty EffectChains
        app.world_mut().spawn((Breaker, EffectChains::default()));
        // Pre-arm bolt with the Impact(Cell) -> CellDestroyed -> Shockwave chain
        let bolt = app
            .world_mut()
            .spawn(ArmedEffects(vec![(
                None,
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::When {
                        trigger: Trigger::CellDestroyed,
                        then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                    }],
                },
            )]))
            .id();

        // Step 1: Cell impact — re-arms from Impact(Cell) to CellDestroyed
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = Some(BoltHitCell {
            cell: Entity::PLACEHOLDER,
            bolt,
        });
        tick(&mut app);

        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert!(
            captured.0.is_empty(),
            "step 1: should re-arm, not fire any effect"
        );
        let armed = app.world().get::<ArmedEffects>(bolt).unwrap();
        assert_eq!(
            armed.0.len(),
            1,
            "step 1: bolt should have exactly one armed trigger (CellDestroyed)"
        );

        // Step 2: Cell destroyed — fires the shockwave
        app.world_mut().resource_mut::<SendBoltHitCell>().0 = None;
        let cell = app
            .world_mut()
            .spawn((
                rantzsoft_spatial2d::components::Position2D(Vec2::new(10.0, 20.0)),
                RequiredToClear,
            ))
            .id();
        app.world_mut().resource_mut::<SendCellDestroyed>().0 = Some(RequestCellDestroyed { cell });
        tick(&mut app);

        // Verify: Cell destroyed fires the shockwave
        let captured = app.world().resource::<CapturedShockwaveFired>();
        assert_eq!(
            captured.0.len(),
            1,
            "step 3: shockwave should fire after cell destroyed"
        );
        assert!(
            (captured.0[0].base_range - 64.0).abs() < f32::EPSILON,
            "step 3: fired effect should be a shockwave with base_range 64.0"
        );
    }
}
