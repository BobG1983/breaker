//! Tests for `BoundEffects` and `StagedEffects` component insertion on cells and bolts.

use bevy::prelude::*;

use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Behavior 11: BoundEffects and StagedEffects inserted if absent on self-targeted cell ──

#[test]
fn bound_effects_and_staged_effects_inserted_on_cell_if_absent() {
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
    // Spawn cell with NO BoundEffects and NO StagedEffects
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    app.update();

    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_some(),
        "BoundEffects should be inserted on cell when absent"
    );
    assert!(
        app.world().get::<StagedEffects>(cell_entity).is_some(),
        "StagedEffects should be inserted on cell when absent"
    );
}

// ── Behavior 11 edge case: Cell has BoundEffects but no StagedEffects ──

#[test]
fn staged_effects_inserted_when_bound_effects_already_exists() {
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
    // Spawn cell WITH BoundEffects but WITHOUT StagedEffects
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias('E'), BoundEffects::default()))
        .id();
    app.update();

    assert!(
        app.world().get::<StagedEffects>(cell_entity).is_some(),
        "StagedEffects should be inserted even when BoundEffects already existed"
    );
}

// ── Behavior 12: Non-Cell target entities get BoundEffects/StagedEffects pre-inserted ──

#[test]
fn bolt_gets_bound_effects_and_staged_effects_pre_inserted() {
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
    app.world_mut().spawn((Cell, CellTypeAlias('B')));
    // Bolt spawned with NO BoundEffects, NO StagedEffects
    let bolt_entity = app.world_mut().spawn(Bolt).id();
    app.update();

    assert!(
        app.world().get::<BoundEffects>(bolt_entity).is_some(),
        "bolt should have BoundEffects pre-inserted by dispatch"
    );
    assert!(
        app.world().get::<StagedEffects>(bolt_entity).is_some(),
        "bolt should have StagedEffects pre-inserted by dispatch"
    );
}

// ── Behavior 12 edge case: Bolt has BoundEffects but not StagedEffects ──

#[test]
fn bolt_with_bound_effects_but_no_staged_effects_gets_staged_effects_inserted() {
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
    app.world_mut().spawn((Cell, CellTypeAlias('B')));
    // Bolt has BoundEffects but NOT StagedEffects
    let bolt_entity = app.world_mut().spawn((Bolt, BoundEffects::default())).id();
    app.update();

    assert!(
        app.world().get::<StagedEffects>(bolt_entity).is_some(),
        "StagedEffects should be inserted on bolt even when BoundEffects already existed"
    );
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
