//! Regression tests: `Target::Bolt` and `Target::Wall` dispatch to ALL matching
//! entities, not just the first.

use bevy::prelude::*;

use super::super::helpers::{make_cell_def, test_app};
use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
    walls::components::Wall,
};

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
