//! Tests for `StampTarget::Breaker` effect dispatch to breaker entities.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect_v3::{
        effects::QuickStopConfig,
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 6: Cell with StampTarget::Breaker dispatches to breaker entity ──

#[test]
fn cell_with_target_breaker_dispatches_to_breaker_entity() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "R".to_owned(),
        make_cell_def(
            "breaker_buff_cell",
            "R",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::QuickStop(QuickStopConfig {
                        multiplier: OrderedFloat(2.0),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("R".to_owned())))
        .id();
    let breaker_entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert(BoundEffects::default());
    app.update();

    // Cell should have CellEffectsDispatched
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker"
    );

    // Breaker should have 1 entry in BoundEffects
    let breaker_bound = app
        .world()
        .get::<BoundEffects>(breaker_entity)
        .expect("breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "breaker should have 1 BoundEffects entry"
    );
    let (chip_name, node) = &breaker_bound.0[0];
    assert_eq!(chip_name, "", "chip_name should be empty string");
    assert!(
        matches!(
            node,
            Tree::When(
                Trigger::Bumped,
                inner,
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::QuickStop(QuickStopConfig { multiplier })) if (multiplier.0 - 2.0).abs() < f32::EPSILON)
        ),
        "expected When(Bumped, Fire(QuickStop {{ multiplier: 2.0 }})), got {node:?}"
    );

    // Cell itself should NOT get these effects
    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_none(),
        "cell should NOT have BoundEffects from breaker-targeted effect"
    );
}

// ── Behavior 6 edge case: No breaker present ──

#[test]
fn cell_with_target_breaker_no_breaker_present_no_panic() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "R".to_owned(),
        make_cell_def(
            "breaker_buff_cell",
            "R",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::QuickStop(QuickStopConfig {
                        multiplier: OrderedFloat(2.0),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("R".to_owned())))
        .id();
    // No breaker entity spawned
    app.update();

    // Cell should still get CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched even with no breaker present"
    );
}
