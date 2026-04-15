//! Tests for `StampTarget::Bolt` effect dispatch to bolt entities.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    cells::{
        components::{CellEffectsDispatched, CellTypeAlias},
        resources::CellTypeRegistry,
    },
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 5: Cell with StampTarget::Bolt effect dispatches to bolt entity ──

#[test]
fn cell_with_target_bolt_dispatches_to_bolt_entity() {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "B".to_owned(),
        make_cell_def(
            "bolt_boost_cell",
            "B",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("B".to_owned())))
        .id();
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
            Tree::When(
                Trigger::Bumped,
                inner,
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 1.2).abs() < f32::EPSILON)
        ),
        "expected When(Bumped, Fire(SpeedBoost {{ multiplier: 1.2 }})), got {node:?}"
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
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "B".to_owned(),
        make_cell_def(
            "bolt_boost_cell",
            "B",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("B".to_owned())))
        .id();
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
