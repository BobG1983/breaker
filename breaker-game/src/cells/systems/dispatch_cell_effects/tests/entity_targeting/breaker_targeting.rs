//! Tests for `Target::Breaker` effect dispatch to breaker entities.

use bevy::prelude::*;

use super::super::helpers::{make_cell_def, test_app};
use crate::{
    breaker::components::Breaker,
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
};

// ── Behavior 6: Cell with Target::Breaker dispatches to breaker entity ──

#[test]
fn cell_with_target_breaker_dispatches_to_breaker_entity() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'R',
        make_cell_def(
            "breaker_buff_cell",
            'R',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::QuickStop { multiplier: 2.0 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('R'))).id();
    let def = crate::breaker::definition::BreakerDefinition::default();
    let breaker_entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert((BoundEffects::default(), StagedEffects::default()));
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
            EffectNode::When {
                trigger: Trigger::Bump,
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::QuickStop { multiplier }) if (multiplier - 2.0).abs() < f32::EPSILON)
        ),
        "expected When {{ Bump, [Do(QuickStop {{ multiplier: 2.0 }})] }}, got {node:?}"
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
        'R',
        make_cell_def(
            "breaker_buff_cell",
            'R',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::QuickStop { multiplier: 2.0 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('R'))).id();
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
