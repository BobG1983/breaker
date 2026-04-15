//! Regression tests: `StampTarget::Bolt` and `StampTarget::ActiveWalls` dispatch to ALL matching
//! entities, not just the first.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    cells::components::CellTypeAlias,
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
    state::run::node::systems::dispatch_cell_effects::tests::helpers::{make_cell_def, test_app},
};

// ── Regression: StampTarget::Bolt dispatches to ALL bolt entities, not just first ──

#[test]
fn cell_with_target_bolt_dispatches_to_all_bolt_entities() {
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
                        multiplier: OrderedFloat(1.4),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let _cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("B".to_owned())))
        .id();
    let bolt_a = app.world_mut().spawn(Bolt).id();
    let bolt_b = app.world_mut().spawn(Bolt).id();
    app.update();

    // BOTH bolts should have BoundEffects with 1 entry each
    let bound_a = app
        .world()
        .get::<BoundEffects>(bolt_a)
        .expect("bolt A should have BoundEffects from StampTarget::Bolt dispatch");
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
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 1.4).abs() < f32::EPSILON)
        ),
        "bolt A expected When(Bumped, Fire(SpeedBoost {{ multiplier: 1.4 }})), got {node_a:?}"
    );

    let bound_b = app
        .world()
        .get::<BoundEffects>(bolt_b)
        .expect("bolt B should have BoundEffects from StampTarget::Bolt dispatch");
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
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 1.4).abs() < f32::EPSILON)
        ),
        "bolt B expected When(Bumped, Fire(SpeedBoost {{ multiplier: 1.4 }})), got {node_b:?}"
    );
}

// ── Regression: StampTarget::ActiveWalls dispatches to ALL wall entities, not just first ──

#[test]
fn cell_with_target_wall_dispatches_to_all_wall_entities() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "W".to_owned(),
        make_cell_def(
            "wall_buff_cell",
            "W",
            10.0,
            Some(vec![RootNode::Stamp(
                StampTarget::ActiveWalls,
                Tree::When(
                    Trigger::Impacted(EntityKind::Bolt),
                    Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(0.8),
                    }))),
                ),
            )]),
        ),
    );

    let mut app = test_app(registry);
    let _cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("W".to_owned())))
        .id();
    let wall_a = app.world_mut().spawn(Wall).id();
    let wall_b = app.world_mut().spawn(Wall).id();
    app.update();

    // BOTH walls should have BoundEffects with 1 entry each
    let bound_a = app
        .world()
        .get::<BoundEffects>(wall_a)
        .expect("wall A should have BoundEffects from StampTarget::ActiveWalls dispatch");
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
            Tree::When(
                Trigger::Impacted(EntityKind::Bolt),
                inner,
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 0.8).abs() < f32::EPSILON)
        ),
        "wall A expected When(Impacted(Bolt), Fire(SpeedBoost {{ multiplier: 0.8 }})), got {node_a:?}"
    );

    let bound_b = app
        .world()
        .get::<BoundEffects>(wall_b)
        .expect("wall B should have BoundEffects from StampTarget::ActiveWalls dispatch");
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
            Tree::When(
                Trigger::Impacted(EntityKind::Bolt),
                inner,
            ) if matches!(inner.as_ref(), Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig { multiplier })) if (multiplier.0 - 0.8).abs() < f32::EPSILON)
        ),
        "wall B expected When(Impacted(Bolt), Fire(SpeedBoost {{ multiplier: 0.8 }})), got {node_b:?}"
    );
}
