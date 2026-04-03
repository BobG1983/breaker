//! Tests for `walk_bound_node` pushing On children to `StagedEffects` (Behavior 14).

use bevy::prelude::*;

use super::helpers::*;
use crate::effect::core::*;

// ── Behavior 14: walk_bound_node pushes On children to StagedEffects ──

#[test]
fn walk_bound_node_pushes_on_child_to_staged_effects_when_trigger_matches() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let bound_node = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllCells,
            permanent: true,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
            }],
        }],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("cell_fortify".into(), bound_node)]),
            StagedEffects::default(),
        ))
        .id();

    app.add_systems(Update, sys_evaluate_bound_for_node_start);
    app.update();

    // After evaluation, StagedEffects should have 1 entry (the On node)
    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "StagedEffects should have 1 entry (the On node pushed from walk_bound_node)"
    );
    assert_eq!(staged.0[0].0, "cell_fortify", "chip_name preserved");
    assert!(
        matches!(
            &staged.0[0].1,
            EffectNode::On {
                target: Target::AllCells,
                permanent: true,
                then: inner,
            } if inner.len() == 1
        ),
        "Pushed entry should be the On(AllCells, permanent: true, ...) node, got {:?}",
        staged.0[0].1
    );

    // BoundEffects should be unchanged (entries are never consumed)
    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1, "BoundEffects entry must not be consumed");
}

// ── Behavior 14 edge case: On node with multiple children ──

#[test]
fn walk_bound_node_pushes_on_with_multiple_children_as_single_entry() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let bound_node = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: Target::AllBolts,
            permanent: true,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 500.0,
                    })],
                },
            ],
        }],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("bolt_buff".into(), bound_node)]),
            StagedEffects::default(),
        ))
        .id();

    app.add_systems(Update, sys_evaluate_bound_for_node_start);
    app.update();

    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "Entire On node (with both children) should be pushed as a single entry"
    );

    if let EffectNode::On { then: inner, .. } = &staged.0[0].1 {
        assert_eq!(
            inner.len(),
            2,
            "On node should have 2 children (both When nodes)"
        );
    } else {
        panic!("Expected On(...) in StagedEffects, got {:?}", staged.0[0].1);
    }
}
