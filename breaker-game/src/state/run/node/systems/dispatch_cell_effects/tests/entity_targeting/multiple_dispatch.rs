//! Tests for multiple cells dispatched independently.

use bevy::prelude::*;

use crate::{
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
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

    // Cell A: has BoundEffects with 1 entry
    let bound_a = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("Cell A should have BoundEffects");
    assert_eq!(
        bound_a.0.len(),
        1,
        "Cell A should have 1 BoundEffects entry"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_a).is_some(),
        "Cell A should have CellEffectsDispatched"
    );

    // Cell B: has BoundEffects with 1 entry
    let bound_b = app
        .world()
        .get::<BoundEffects>(cell_b)
        .expect("Cell B should have BoundEffects");
    assert_eq!(
        bound_b.0.len(),
        1,
        "Cell B should have 1 BoundEffects entry"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_b).is_some(),
        "Cell B should have CellEffectsDispatched"
    );

    // Cell C: no effects
    assert!(
        app.world().get::<BoundEffects>(cell_c).is_none(),
        "Cell C should NOT have BoundEffects"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_c).is_none(),
        "Cell C should NOT have CellEffectsDispatched"
    );
}
