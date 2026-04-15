//! Tests for `StampTarget::ActiveBolts` and `StampTarget::ActiveCells` broadcast dispatch.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    cells::{
        components::{CellEffectsDispatched, CellTypeAlias},
        resources::CellTypeRegistry,
    },
    effect_v3::{
        effects::{ExplodeConfig, SpeedBoostConfig},
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── StampTarget::ActiveBolts dispatches to all bolt entities ──

#[test]
fn cell_with_target_all_bolts_dispatches_to_all_bolt_entities() {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "B".to_owned(),
        make_cell_def(
            "all_bolts_cell",
            "B",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::ActiveBolts,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
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
    let bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();
    app.update();

    // Both bolts should have BoundEffects with 1 entry each
    let bound_a = app
        .world()
        .get::<BoundEffects>(bolt_a)
        .expect("bolt A should have BoundEffects from ActiveBolts dispatch");
    assert_eq!(
        bound_a.0.len(),
        1,
        "bolt A should have exactly 1 BoundEffects entry"
    );
    let (chip_name_a, node_a) = &bound_a.0[0];
    assert_eq!(
        chip_name_a, "",
        "chip_name should be empty string for cell-defined effects"
    );
    assert!(
        matches!(
            node_a,
            Tree::When(
                Trigger::Bumped,
                inner,
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 1.5).abs() < f32::EPSILON)
        ),
        "bolt A expected When(Bumped, Fire(SpeedBoost {{ multiplier: 1.5 }})), got {node_a:?}"
    );

    let bound_b = app
        .world()
        .get::<BoundEffects>(bolt_b)
        .expect("bolt B should have BoundEffects from ActiveBolts dispatch");
    assert_eq!(
        bound_b.0.len(),
        1,
        "bolt B should have exactly 1 BoundEffects entry"
    );
    let (chip_name_b, node_b) = &bound_b.0[0];
    assert_eq!(
        chip_name_b, "",
        "chip_name should be empty string for cell-defined effects"
    );
    assert!(
        matches!(
            node_b,
            Tree::When(
                Trigger::Bumped,
                inner,
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 1.5).abs() < f32::EPSILON)
        ),
        "bolt B expected When(Bumped, Fire(SpeedBoost {{ multiplier: 1.5 }})), got {node_b:?}"
    );

    // Cell should have CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker"
    );

    // Cell itself should NOT get BoundEffects (effect targets bolts, not cell)
    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_none(),
        "cell should NOT have BoundEffects from ActiveBolts-targeted effect"
    );
}

// ── Behavior 7: Cell with StampTarget::ActiveCells dispatches to ALL cell entities ──

#[test]
fn cell_with_target_all_cells_dispatches_to_all_cells() {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "A".to_owned(),
        make_cell_def(
            "all_cells_buff",
            "A",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                        range:  OrderedFloat(32.0),
                        damage: OrderedFloat(0.5),
                    }))),
                ),
            )]),
        ),
    );
    registry.insert("S".to_owned(), make_cell_def("standard", "S", 10.0, None));

    let mut app = test_app(registry);
    let cell_a = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("A".to_owned())))
        .id();
    let cell_b = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("S".to_owned())))
        .id();
    app.update();

    // Cell A (source) has BoundEffects with 1 entry (ActiveCells includes self)
    let bound_a = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("Cell A should have BoundEffects (ActiveCells includes source)");
    assert_eq!(
        bound_a.0.len(),
        1,
        "Cell A should have 1 BoundEffects entry from ActiveCells"
    );

    // Cell B (other cell) also has BoundEffects with 1 entry
    let bound_b = app
        .world()
        .get::<BoundEffects>(cell_b)
        .expect("Cell B should have BoundEffects from ActiveCells dispatch");
    assert_eq!(
        bound_b.0.len(),
        1,
        "Cell B should have 1 BoundEffects entry from ActiveCells"
    );

    // Cell A has CellEffectsDispatched marker (it was the source)
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_a).is_some(),
        "Cell A (source) should have CellEffectsDispatched"
    );

    // Cell B does NOT have CellEffectsDispatched (it has no effects of its own)
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_b).is_none(),
        "Cell B should NOT have CellEffectsDispatched (marker is for source cell only)"
    );
}

// ── Behavior 7 edge case: Only 1 cell entity (source gets its own ActiveCells effect) ──

#[test]
fn single_cell_with_all_cells_targets_itself() {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "A".to_owned(),
        make_cell_def(
            "all_cells_buff",
            "A",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Died,
                    Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                        range:  OrderedFloat(32.0),
                        damage: OrderedFloat(0.5),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let cell_a = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("A".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("single cell should receive its own ActiveCells effect");
    assert_eq!(
        bound.0.len(),
        1,
        "single cell should have 1 BoundEffects entry from ActiveCells"
    );
}
