//! Tests for `CellEffectsDispatched` marker preventing double-dispatch.

use bevy::prelude::*;

use super::super::helpers::{make_cell_def, test_app};
use crate::{
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

// ── Behavior 10: CellEffectsDispatched prevents double-dispatch ──

#[test]
fn cell_effects_dispatched_marker_prevents_double_dispatch() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'E',
        make_cell_def(
            "effect_cell",
            'E',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Cell,
                then: vec![EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 48.0,
                        damage: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    // Spawn cell that already has the marker and 1 existing entry
    let cell_entity = app
        .world_mut()
        .spawn((
            Cell,
            CellTypeAlias('E'),
            CellEffectsDispatched,
            BoundEffects(vec![(
                String::new(),
                EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 48.0,
                        damage: 1.0,
                    })],
                },
            )]),
        ))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should still have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should still have 1 entry (no double-dispatch), got {}",
        bound.0.len()
    );
}

// ── Behavior 10 edge case: Marker on A (skipped), no marker on B (dispatched) ──

#[test]
fn marker_on_one_cell_skips_it_while_other_is_dispatched() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'E',
        make_cell_def(
            "effect_cell",
            'E',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Cell,
                then: vec![EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 48.0,
                        damage: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    // Cell A: already dispatched (has marker)
    let cell_a = app
        .world_mut()
        .spawn((
            Cell,
            CellTypeAlias('E'),
            CellEffectsDispatched,
            BoundEffects(vec![(
                String::new(),
                EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 48.0,
                        damage: 1.0,
                    })],
                },
            )]),
        ))
        .id();
    // Cell B: not dispatched yet
    let cell_b = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    app.update();

    // Cell A unchanged (still 1 entry)
    let bound_a = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("Cell A should have BoundEffects");
    assert_eq!(
        bound_a.0.len(),
        1,
        "Cell A should be unchanged (skipped by marker)"
    );

    // Cell B dispatched (now has 1 entry)
    let bound_b = app
        .world()
        .get::<BoundEffects>(cell_b)
        .expect("Cell B should have BoundEffects after dispatch");
    assert_eq!(
        bound_b.0.len(),
        1,
        "Cell B should have 1 BoundEffects entry"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_b).is_some(),
        "Cell B should have CellEffectsDispatched marker"
    );
}
