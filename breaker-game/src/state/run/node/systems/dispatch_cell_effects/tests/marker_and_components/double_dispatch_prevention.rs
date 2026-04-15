//! Tests for `CellEffectsDispatched` marker preventing double-dispatch.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    cells::components::{CellEffectsDispatched, CellTypeAlias},
    effect_v3::{
        effects::ExplodeConfig,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 10: CellEffectsDispatched prevents double-dispatch ──

#[test]
fn cell_effects_dispatched_marker_prevents_double_dispatch() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "E".to_owned(),
        make_cell_def(
            "effect_cell",
            "E",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                        range:  OrderedFloat(48.0),
                        damage: OrderedFloat(1.0),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    // Spawn cell that already has the marker and 1 existing entry
    let cell_entity = app
        .world_mut()
        .spawn((
            Cell,
            CellTypeAlias("E".to_owned()),
            CellEffectsDispatched,
            BoundEffects(vec![(
                String::new(),
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                        range:  OrderedFloat(48.0),
                        damage: OrderedFloat(1.0),
                    }))),
                ),
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
        "E".to_owned(),
        make_cell_def(
            "effect_cell",
            "E",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                        range:  OrderedFloat(48.0),
                        damage: OrderedFloat(1.0),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    // Cell A: already dispatched (has marker)
    let cell_a = app
        .world_mut()
        .spawn((
            Cell,
            CellTypeAlias("E".to_owned()),
            CellEffectsDispatched,
            BoundEffects(vec![(
                String::new(),
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                        range:  OrderedFloat(48.0),
                        damage: OrderedFloat(1.0),
                    }))),
                ),
            )]),
        ))
        .id();
    // Cell B: not dispatched yet
    let cell_b = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("E".to_owned())))
        .id();
    app.update();

    // Cell A: gets 1 additional entry from Cell B's dispatch (ActiveCells targets all cells)
    let bound_a = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("Cell A should have BoundEffects");
    assert_eq!(
        bound_a.0.len(),
        2,
        "Cell A should have 2 entries (1 existing + 1 from Cell B's ActiveCells dispatch)"
    );

    // Cell B dispatched (now has 1 entry from its own dispatch to ActiveCells)
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
