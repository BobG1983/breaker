use bevy::prelude::*;

use super::helpers::{make_cell_def, test_app};
use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
    wall::components::Wall,
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
                        damage_mult: 1.0,
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
                        damage_mult: 1.0,
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
                        damage_mult: 1.0,
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
                        damage_mult: 1.0,
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
                        damage_mult: 1.0,
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
                        damage_mult: 1.0,
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

// ── Regression: Target::Bolt dispatches to ALL bolt entities, not just first ──

#[test]
fn cell_with_target_bolt_dispatches_to_all_bolt_entities() {
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
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.4 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let _cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('B'))).id();
    let bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();
    app.update();

    // BOTH bolts should have BoundEffects with 1 entry each
    let bound_a = app
        .world()
        .get::<BoundEffects>(bolt_a)
        .expect("bolt A should have BoundEffects from Target::Bolt dispatch");
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
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.4).abs() < f32::EPSILON)
        ),
        "bolt A expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.4 }})] }}, got {node_a:?}"
    );

    let bound_b = app
        .world()
        .get::<BoundEffects>(bolt_b)
        .expect("bolt B should have BoundEffects from Target::Bolt dispatch");
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
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 1.4).abs() < f32::EPSILON)
        ),
        "bolt B expected When {{ Bumped, [Do(SpeedBoost {{ multiplier: 1.4 }})] }}, got {node_b:?}"
    );
}

// ── Regression: Target::Wall dispatches to ALL wall entities, not just first ──

#[test]
fn cell_with_target_wall_dispatches_to_all_wall_entities() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'W',
        make_cell_def(
            "wall_buff_cell",
            'W',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Wall,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 0.8 })],
                }],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let _cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('W'))).id();
    let wall_a = app.world_mut().spawn(Wall).id();
    let wall_b = app.world_mut().spawn(Wall).id();
    app.update();

    // BOTH walls should have BoundEffects with 1 entry each
    let bound_a = app
        .world()
        .get::<BoundEffects>(wall_a)
        .expect("wall A should have BoundEffects from Target::Wall dispatch");
    assert_eq!(
        bound_a.0.len(),
        1,
        "wall A should have exactly 1 BoundEffects entry"
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
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 0.8).abs() < f32::EPSILON)
        ),
        "wall A expected When {{ Impacted(Bolt), [Do(SpeedBoost {{ multiplier: 0.8 }})] }}, got {node_a:?}"
    );

    let bound_b = app
        .world()
        .get::<BoundEffects>(wall_b)
        .expect("wall B should have BoundEffects from Target::Wall dispatch");
    assert_eq!(
        bound_b.0.len(),
        1,
        "wall B should have exactly 1 BoundEffects entry"
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
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then,
            } if then.len() == 1 && matches!(then[0], EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (multiplier - 0.8).abs() < f32::EPSILON)
        ),
        "wall B expected When {{ Impacted(Bolt), [Do(SpeedBoost {{ multiplier: 0.8 }})] }}, got {node_b:?}"
    );
}
