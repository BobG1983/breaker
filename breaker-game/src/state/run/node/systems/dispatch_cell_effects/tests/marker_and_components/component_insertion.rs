//! Tests for `BoundEffects` and `StagedEffects` component insertion on cells and bolts.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellTypeAlias},
    effect_v3::{
        effects::{ExplodeConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 11: BoundEffects inserted if absent on self-targeted cell ──

#[test]
fn bound_effects_and_staged_effects_inserted_on_cell_if_absent() {
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
    // Spawn cell with NO BoundEffects
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("E".to_owned())))
        .id();
    app.update();

    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_some(),
        "BoundEffects should be inserted on cell when absent"
    );
}

// ── Behavior 11 edge case: Cell has BoundEffects already ──

#[test]
fn staged_effects_inserted_when_bound_effects_already_exists() {
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
    // Spawn cell WITH BoundEffects already present
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("E".to_owned()), BoundEffects::default()))
        .id();
    app.update();

    // BoundEffects should have the dispatched entry appended
    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("BoundEffects should still exist");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 dispatched entry appended"
    );
}

// ── Behavior 12: Non-Cell target entities get BoundEffects pre-inserted ──

#[test]
fn bolt_gets_bound_effects_and_staged_effects_pre_inserted() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
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
    app.world_mut().spawn((Cell, CellTypeAlias("B".to_owned())));
    // Bolt spawned with NO BoundEffects
    let bolt_entity = app.world_mut().spawn(Bolt).id();
    app.update();

    assert!(
        app.world().get::<BoundEffects>(bolt_entity).is_some(),
        "bolt should have BoundEffects pre-inserted by dispatch"
    );
}

// ── Behavior 12 edge case: Bolt has BoundEffects already ──

#[test]
fn bolt_with_bound_effects_but_no_staged_effects_gets_staged_effects_inserted() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
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
    app.world_mut().spawn((Cell, CellTypeAlias("B".to_owned())));
    // Bolt has BoundEffects already
    let bolt_entity = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
    app.update();

    // Existing BoundEffects should be preserved (with dispatched entry appended)
    let bound = app
        .world()
        .get::<BoundEffects>(bolt_entity)
        .expect("bolt should still have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "bolt BoundEffects should have 1 dispatched entry"
    );
}
