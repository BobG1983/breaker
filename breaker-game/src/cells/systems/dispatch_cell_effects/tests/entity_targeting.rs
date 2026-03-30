use bevy::prelude::*;

use super::helpers::{make_cell_def, test_app};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{
        BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger,
    },
};

// ── Behavior 4: Multiple cells each get their own effects dispatched independently ──

#[test]
fn multiple_cells_dispatched_independently() {
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
    registry.insert('S', make_cell_def("standard", 'S', 10.0, None));

    let mut app = test_app(registry);
    let cell_a = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    let cell_b = app.world_mut().spawn((Cell, CellTypeAlias('E'))).id();
    let cell_c = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
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

// ── Behavior 5: Cell with Target::Bolt effect dispatches to bolt entity ──

#[test]
fn cell_with_target_bolt_dispatches_to_bolt_entity() {
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
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
    let bolt_entity = app.world_mut().spawn(Bolt).id();
    app.update();

    // Cell should have CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker"
    );

    // Bolt should have BoundEffects with 1 entry
    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt_entity)
        .expect("bolt should have BoundEffects after dispatch");
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "bolt should have 1 BoundEffects entry"
    );
    let (chip_name, node) = &bolt_bound.0[0];
    assert_eq!(chip_name, "", "chip_name should be empty string");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Bumped,
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.2).abs() < f32::EPSILON)
        ),
        "expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.2 }})] }}, got {node:?}"
    );

    // Bolt should have StagedEffects
    assert!(
        app.world().get::<StagedEffects>(bolt_entity).is_some(),
        "bolt should have StagedEffects after dispatch"
    );

    // Cell itself should NOT get BoundEffects from bolt-targeted effect
    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_none(),
        "cell should NOT have BoundEffects from bolt-targeted effect"
    );
}

// ── Behavior 5 edge case: No bolt present ──

#[test]
fn cell_with_target_bolt_no_bolt_present_no_panic() {
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
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
    // No bolt entity spawned
    app.update();

    // Cell should still get CellEffectsDispatched marker
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched marker even with no bolt present"
    );

    // Cell should NOT get BoundEffects (effect targets bolt, not cell)
    assert!(
        app.world().get::<BoundEffects>(cell_entity).is_none(),
        "cell should NOT have BoundEffects when bolt-targeted and no bolt present"
    );
}

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
    let breaker_entity = app
        .world_mut()
        .spawn((Breaker, BoundEffects::default(), StagedEffects::default()))
        .id();
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

// ── Target::AllBolts dispatches to all bolt entities ──

#[test]
fn cell_with_target_all_bolts_dispatches_to_all_bolt_entities() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'B',
        make_cell_def(
            "all_bolts_cell",
            'B',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
    let bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();
    app.update();

    // Both bolts should have BoundEffects with 1 entry each
    let bound_a = app
        .world()
        .get::<BoundEffects>(bolt_a)
        .expect("bolt A should have BoundEffects from AllBolts dispatch");
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
            EffectNode::When {
                trigger: Trigger::Bumped,
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.5).abs() < f32::EPSILON)
        ),
        "bolt A expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.5 }})] }}, got {node_a:?}"
    );

    let bound_b = app
        .world()
        .get::<BoundEffects>(bolt_b)
        .expect("bolt B should have BoundEffects from AllBolts dispatch");
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
            EffectNode::When {
                trigger: Trigger::Bumped,
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.5).abs() < f32::EPSILON)
        ),
        "bolt B expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.5 }})] }}, got {node_b:?}"
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
        "cell should NOT have BoundEffects from AllBolts-targeted effect"
    );
}

// ── Behavior 7: Cell with Target::AllCells dispatches to ALL cell entities ──

#[test]
fn cell_with_target_all_cells_dispatches_to_all_cells() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'A',
        make_cell_def(
            "all_cells_buff",
            'A',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 32.0,
                        damage_mult: 0.5,
                    })],
                }],
            }]),
        ),
    );
    registry.insert('S', make_cell_def("standard", 'S', 10.0, None));

    let mut app = test_app(registry);
    let cell_a = app.world_mut().spawn((Cell, CellTypeAlias('A'))).id();
    let cell_b = app.world_mut().spawn((Cell, CellTypeAlias('S'))).id();
    app.update();

    // Cell A (source) has BoundEffects with 1 entry (AllCells includes self)
    let bound_a = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("Cell A should have BoundEffects (AllCells includes source)");
    assert_eq!(
        bound_a.0.len(),
        1,
        "Cell A should have 1 BoundEffects entry from AllCells"
    );

    // Cell B (other cell) also has BoundEffects with 1 entry
    let bound_b = app
        .world()
        .get::<BoundEffects>(cell_b)
        .expect("Cell B should have BoundEffects from AllCells dispatch");
    assert_eq!(
        bound_b.0.len(),
        1,
        "Cell B should have 1 BoundEffects entry from AllCells"
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

// ── Behavior 7 edge case: Only 1 cell entity (source gets its own AllCells effect) ──

#[test]
fn single_cell_with_all_cells_targets_itself() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'A',
        make_cell_def(
            "all_cells_buff",
            'A',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 32.0,
                        damage_mult: 0.5,
                    })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_a = app.world_mut().spawn((Cell, CellTypeAlias('A'))).id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(cell_a)
        .expect("single cell should receive its own AllCells effect");
    assert_eq!(
        bound.0.len(),
        1,
        "single cell should have 1 BoundEffects entry from AllCells"
    );
}
