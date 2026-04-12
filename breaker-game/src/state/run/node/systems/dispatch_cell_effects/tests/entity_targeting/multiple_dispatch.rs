//! Tests for multiple cells dispatched independently.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect_v3::{
        effects::ExplodeConfig,
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 4: Multiple cells each get their own effects dispatched independently ──

#[test]
fn multiple_cells_dispatched_independently() {
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
                        range: OrderedFloat(48.0),
                        damage: OrderedFloat(1.0),
                    }))),
                ),
            )]),
        ),
    );
    registry.insert("S".to_owned(), make_cell_def("standard", "S", 10.0, None));

    let mut app = test_app(registry);
    let cell_a = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("E".to_owned())))
        .id();
    let cell_b = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("E".to_owned())))
        .id();
    let cell_c = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("S".to_owned())))
        .id();
    app.update();

    // Cell A: has BoundEffects (ActiveCells dispatches to all cells including A and B)
    let bound_a = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("Cell A should have BoundEffects");
    // Both cell A and cell B dispatch to all cells via ActiveCells, so each cell gets 2 entries
    assert_eq!(
        bound_a.0.len(),
        2,
        "Cell A should have 2 BoundEffects entries (one from each E cell)"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_a).is_some(),
        "Cell A should have CellEffectsDispatched"
    );

    // Cell B: has BoundEffects with 2 entries
    let bound_b = app
        .world()
        .get::<BoundEffects>(cell_b)
        .expect("Cell B should have BoundEffects");
    assert_eq!(
        bound_b.0.len(),
        2,
        "Cell B should have 2 BoundEffects entries (one from each E cell)"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_b).is_some(),
        "Cell B should have CellEffectsDispatched"
    );

    // Cell C: also gets effects from both E cells (ActiveCells targets ALL cells)
    let bound_c = app
        .world()
        .get::<BoundEffects>(cell_c)
        .expect("Cell C should have BoundEffects from ActiveCells dispatch");
    assert_eq!(
        bound_c.0.len(),
        2,
        "Cell C should have 2 BoundEffects entries (one from each E cell)"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_c).is_none(),
        "Cell C should NOT have CellEffectsDispatched (it has no effects of its own)"
    );
}
