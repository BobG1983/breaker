use bevy::prelude::*;

use super::helpers::{make_cell_def, test_app};
use crate::{
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
};

// ── Behavior 1: Cell with effects gets children pushed to BoundEffects (Target::Cell) ──

#[test]
fn cell_with_target_cell_effect_gets_bound_effects_populated() {
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
                        damage_mult: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    app.update();

    // Cell should have BoundEffects with exactly 1 entry
    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should have BoundEffects after dispatch");
    assert_eq!(
        bound.0.len(),
        1,
        "cell should have exactly 1 BoundEffects entry, got {}",
        bound.0.len()
    );
    let (chip_name, node) = &bound.0[0];
    assert_eq!(
        chip_name, "",
        "chip_name should be empty string for cell-defined effects"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Died,
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::Explode { range, damage_mult }) if (range - 48.0).abs() < f32::EPSILON && (damage_mult - 1.0).abs() < f32::EPSILON)
        ),
        "expected When {{ Died, [Do(Explode {{ range: 48.0, damage_mult: 1.0 }})] }}, got {node:?}"
    );

    // Cell should have StagedEffects (default-inserted)
    assert!(
        app.world().get::<StagedEffects>(cell_entity).is_some(),
        "cell should have StagedEffects after dispatch"
    );

    // Cell should have CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker after dispatch"
    );
}

// ── Behavior 1 edge case: Cell with existing BoundEffects but no marker ──

#[test]
fn cell_with_existing_bound_effects_but_no_marker_still_gets_dispatched() {
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
                        damage_mult: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((
            Cell,
            CellTypeAlias('E'),
            BoundEffects(vec![(
                "existing_chip".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                },
            )]),
        ))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should have BoundEffects after dispatch");
    assert_eq!(
        bound.0.len(),
        2,
        "cell should have 2 BoundEffects entries (1 existing + 1 dispatched), got {}",
        bound.0.len()
    );
    // Existing entry at index 0 is preserved
    assert_eq!(
        bound.0[0].0, "existing_chip",
        "existing entry at index 0 should be preserved"
    );
    // Dispatched entry at index 1
    assert_eq!(
        bound.0[1].0, "",
        "dispatched entry at index 1 should have empty chip_name"
    );

    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker"
    );
}

// ── Behavior 2: Cell with no effects is unchanged ──

#[test]
fn cell_with_no_effects_is_unchanged() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert('S', make_cell_def("standard", 'S', 10.0, None));
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
                        damage_mult: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_s = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
    let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    app.update();

    // Positive: 'E' cell with effects SHOULD get BoundEffects
    let bound_e = app
        .world()
        .get::<BoundEffects>(cell_e)
        .expect("cell 'E' with effects should have BoundEffects after dispatch");
    assert_eq!(
        bound_e.0.len(),
        1,
        "cell 'E' should have exactly 1 BoundEffects entry"
    );

    // Negative: 'S' cell with no effects should NOT get BoundEffects
    assert!(
        app.world().get::<BoundEffects>(cell_s).is_none(),
        "cell 'S' with no effects should NOT have BoundEffects"
    );
    assert!(
        app.world().get::<StagedEffects>(cell_s).is_none(),
        "cell 'S' with no effects should NOT have StagedEffects"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_s).is_none(),
        "cell 'S' with no effects should NOT have CellEffectsDispatched"
    );
}

// ── Behavior 2 edge case: effects is Some(vec![]) (empty vec) ──

#[test]
fn cell_with_empty_effects_vec_is_unchanged() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert('S', make_cell_def("standard", 'S', 10.0, Some(vec![])));
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
                        damage_mult: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_s = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
    let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    app.update();

    // Positive: 'E' cell with non-empty effects SHOULD get BoundEffects
    let bound_e = app
        .world()
        .get::<BoundEffects>(cell_e)
        .expect("cell 'E' with non-empty effects should have BoundEffects after dispatch");
    assert_eq!(
        bound_e.0.len(),
        1,
        "cell 'E' should have exactly 1 BoundEffects entry"
    );

    // Negative: 'S' cell with empty effects vec should NOT get BoundEffects
    assert!(
        app.world().get::<BoundEffects>(cell_s).is_none(),
        "cell 'S' with empty effects vec should NOT have BoundEffects"
    );
    assert!(
        app.world().get::<StagedEffects>(cell_s).is_none(),
        "cell 'S' with empty effects vec should NOT have StagedEffects"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_s).is_none(),
        "cell 'S' with empty effects vec should NOT have CellEffectsDispatched"
    );
}

// ── Behavior 3: Cell with alias not found in registry is skipped ──

#[test]
fn cell_with_unknown_alias_is_skipped_no_panic() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert('S', make_cell_def("standard", 'S', 10.0, None));
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
                        damage_mult: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_x = app.world_mut().spawn((Cell, CellTypeAlias('X'))).id();
    let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    app.update();

    // Positive: 'E' cell with known alias and effects SHOULD get BoundEffects
    let bound_e = app
        .world()
        .get::<BoundEffects>(cell_e)
        .expect("cell 'E' with known alias should have BoundEffects after dispatch");
    assert_eq!(
        bound_e.0.len(),
        1,
        "cell 'E' should have exactly 1 BoundEffects entry"
    );

    // Negative: 'X' cell with unknown alias should NOT get BoundEffects
    assert!(
        app.world().get::<BoundEffects>(cell_x).is_none(),
        "cell with unknown alias should NOT have BoundEffects"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_x).is_none(),
        "cell with unknown alias should NOT have CellEffectsDispatched"
    );
}

// ── Behavior 3 edge case: Known alias dispatched, missing alias skipped ──

#[test]
fn cell_with_alias_not_in_registry_skipped_while_known_alias_dispatched() {
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
                        damage_mult: 1.0,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_e = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    let cell_x = app.world_mut().spawn((Cell, CellTypeAlias('X'))).id();
    app.update();

    // Positive: 'E' cell with known alias SHOULD get BoundEffects
    let bound_e = app
        .world()
        .get::<BoundEffects>(cell_e)
        .expect("cell 'E' with known alias should have BoundEffects after dispatch");
    assert_eq!(
        bound_e.0.len(),
        1,
        "cell 'E' should have exactly 1 BoundEffects entry"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_e).is_some(),
        "cell 'E' should have CellEffectsDispatched marker"
    );

    // Negative: 'X' cell with alias not in registry should NOT get BoundEffects
    assert!(
        app.world().get::<BoundEffects>(cell_x).is_none(),
        "cell 'X' with alias not in registry should NOT have BoundEffects"
    );
    assert!(
        app.world().get::<CellEffectsDispatched>(cell_x).is_none(),
        "cell 'X' with alias not in registry should NOT have CellEffectsDispatched"
    );
}
