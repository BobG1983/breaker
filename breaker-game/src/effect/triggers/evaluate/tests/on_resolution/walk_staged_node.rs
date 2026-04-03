//! Tests for `walk_staged_node` handling On nodes via `ResolveOnCommand` (Behavior 15).

use bevy::prelude::*;

use super::helpers::*;
use crate::{cells::components::Cell, effect::core::*};

// ── Behavior 15: On in StagedEffects consumed and children transferred to targets ──

#[test]
fn on_node_in_staged_effects_consumed_and_resolved_to_target_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Source entity with the On node in StagedEffects
    let source = app
        .world_mut()
        .spawn(StagedEffects(vec![(
            "cell_fortify".into(),
            EffectNode::On {
                target: Target::AllCells,
                permanent: true,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield {
                        duration: 5.0,
                        reflection_cost: 0.0,
                    })],
                }],
            },
        )]))
        .id();

    // Target Cell entities
    let cell_a = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();
    let cell_b = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    app.add_systems(Update, sys_evaluate_staged_for_node_start);
    // First update: evaluate_staged_effects runs, queues ResolveOnCommand
    app.update();

    // The On entry should be consumed from source's StagedEffects
    let staged = app.world().get::<StagedEffects>(source).unwrap();
    assert_eq!(
        staged.0.len(),
        0,
        "On node should be consumed from StagedEffects after evaluation"
    );

    // After command application, each Cell should have BoundEffects updated
    for (label, cell) in [("cell_a", cell_a), ("cell_b", cell_b)] {
        let bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 BoundEffects entry after ResolveOnCommand"
        );
        assert_eq!(bound.0[0].0, "cell_fortify", "{label} chip_name preserved");
        assert!(
            matches!(
                &bound.0[0].1,
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    ..
                }
            ),
            "{label} should have When(Impacted(Bolt), ...) in BoundEffects, got {:?}",
            bound.0[0].1
        );
    }
}

// ── Behavior 15 edge case: permanent: false sends to StagedEffects ──

#[test]
fn on_node_with_permanent_false_sends_children_to_staged_effects_on_targets() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let source = app
        .world_mut()
        .spawn(StagedEffects(vec![(
            "cell_fortify".into(),
            EffectNode::On {
                target: Target::AllCells,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield {
                        duration: 5.0,
                        reflection_cost: 0.0,
                    })],
                }],
            },
        )]))
        .id();

    let cell = app
        .world_mut()
        .spawn((Cell, BoundEffects::default(), StagedEffects::default()))
        .id();

    app.add_systems(Update, sys_evaluate_staged_for_node_start);
    app.update();

    // On node consumed from source
    let staged = app.world().get::<StagedEffects>(source).unwrap();
    assert_eq!(staged.0.len(), 0, "On node should be consumed");

    // With permanent: false, children go to StagedEffects (not BoundEffects)
    let cell_staged = app.world().get::<StagedEffects>(cell).unwrap();
    assert_eq!(
        cell_staged.0.len(),
        1,
        "Cell should have 1 StagedEffects entry (permanent: false)"
    );

    let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert!(
        cell_bound.0.is_empty(),
        "Cell BoundEffects should remain empty when permanent: false"
    );
}
