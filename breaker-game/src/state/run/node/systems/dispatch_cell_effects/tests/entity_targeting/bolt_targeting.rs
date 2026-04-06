//! Tests for `Target::Bolt` effect dispatch to bolt entities.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 5: Cell with Target::Bolt effect dispatches to bolt entity ──

#[test]
fn cell_with_target_bolt_dispatches_to_bolt_entity() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'B',
        make_cell_def(
            "bolt_boost_cell",
            'B',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
    let bolt_entity = app.world_mut().spawn(Bolt).id();
    app.update();

    // Cell should have CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker"
    );

    // Bolt should have BoundEffects with 1 entry
    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt_entity)
        .expect("bolt should have BoundEffects after dispatch");
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "bolt should have 1 BoundEffects entry"
    );
    let (chip_name, node) = &bolt_bound.0[0];
    assert_eq!(chip_name, "", "chip_name should be empty string");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Bumped,
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.2).abs() < f32::EPSILON)
        ),
        "expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.2 }})] }}, got {node:?}"
    );

    // Bolt should have StagedEffects
    assert!(
        app.world().get::<StagedEffects>(bolt_entity).is_some(),
        "bolt should have StagedEffects after dispatch"
    );

    // Cell itself should NOT get BoundEffects from bolt-targeted effect
    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_none(),
        "cell should NOT have BoundEffects from bolt-targeted effect"
    );
}

// ── Behavior 5 edge case: No bolt present ──

#[test]
fn cell_with_target_bolt_no_bolt_present_no_panic() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'B',
        make_cell_def(
            "bolt_boost_cell",
            'B',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
    // No bolt entity spawned
    app.update();

    // Cell should still get CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker even with no bolt present"
    );

    // Cell should NOT get BoundEffects (effect targets bolt, not cell)
    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_none(),
        "cell should NOT have BoundEffects when bolt-targeted and no bolt present"
    );
}
