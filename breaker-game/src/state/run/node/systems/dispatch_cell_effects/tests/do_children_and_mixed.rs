use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::{make_cell_def, test_app};
use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect_v3::{
        effects::{DamageBoostConfig, ExplodeConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 8: Fire children are stamped into BoundEffects alongside When children ──

#[test]
fn do_children_are_not_stored_in_bound_effects() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "D".to_owned(),
        make_cell_def(
            "immediate_effect_cell",
            "D",
            10.0,
            Some(vec![
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    })),
                ),
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::When(
                        Trigger::Died,
                        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                            range:  OrderedFloat(48.0),
                            damage: OrderedFloat(1.0),
                        }))),
                    ),
                ),
            ]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("D".to_owned())))
        .id();
    app.update();

    // BoundEffects should have 2 entries (Fire + When), both stored by stamp_effect
    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries (Fire + When); got {}",
        bound.0.len()
    );
    assert!(
        matches!(&bound.0[0].1, Tree::Fire(EffectType::DamageBoost(..))),
        "the first BoundEffects entry should be the Fire(DamageBoost) node, got {:?}",
        bound.0[0].1
    );
    assert!(
        matches!(&bound.0[1].1, Tree::When(Trigger::Died, ..)),
        "the second BoundEffects entry should be the When(Died) node, got {:?}",
        bound.0[1].1
    );
}

// ── Behavior 8 edge case: All children are Fire nodes ──

#[test]
fn all_do_children_results_in_no_bound_effects_entries() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "D".to_owned(),
        make_cell_def(
            "all_do_cell",
            "D",
            10.0,
            Some(vec![
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    })),
                ),
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                        multiplier: OrderedFloat(1.1),
                    })),
                ),
            ]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("D".to_owned())))
        .id();
    app.update();

    // Both Fire nodes are stamped into BoundEffects
    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("BoundEffects should be present when all children are Fire nodes");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries (both Fire nodes stamped), got {}",
        bound.0.len()
    );

    // Cell should still have marker since it had effects
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched even when all children are Fire nodes"
    );
}

// ── Behavior 9: Cell with multiple RootEffects gets all dispatched ──

#[test]
fn cell_with_multiple_root_effects_gets_all_dispatched() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "M".to_owned(),
        make_cell_def(
            "multi_effect_cell",
            "M",
            20.0,
            Some(vec![
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::When(
                        Trigger::Died,
                        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                            range:  OrderedFloat(48.0),
                            damage: OrderedFloat(1.0),
                        }))),
                    ),
                ),
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::When(
                        Trigger::Impacted(EntityKind::Bolt),
                        Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                            multiplier: OrderedFloat(0.5),
                        }))),
                    ),
                ),
            ]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("M".to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        2,
        "cell should have 2 BoundEffects entries, got {}",
        bound.0.len()
    );

    // Both entries should have empty chip_name
    assert_eq!(bound.0[0].0, "", "first entry chip_name should be empty");
    assert_eq!(bound.0[1].0, "", "second entry chip_name should be empty");

    // First: Died->Explode
    assert!(
        matches!(&bound.0[0].1, Tree::When(Trigger::Died, ..)),
        "first entry should be When(Died), got {:?}",
        bound.0[0].1
    );

    // Second: Impacted(Bolt)->DamageBoost
    assert!(
        matches!(
            &bound.0[1].1,
            Tree::When(Trigger::Impacted(EntityKind::Bolt), ..)
        ),
        "second entry should be When(Impacted(Bolt)), got {:?}",
        bound.0[1].1
    );
}

// ── Behavior 9 edge case: Mix of StampTarget::ActiveCells and StampTarget::Bolt ──

#[test]
fn cell_with_mixed_targets_dispatches_to_correct_entities() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        "M".to_owned(),
        make_cell_def(
            "mixed_target_cell",
            "M",
            10.0,
            Some(vec![
                RootNode::Stamp(
                    StampTarget::ActiveCells,
                    Tree::When(
                        Trigger::Died,
                        Box::new(Tree::Fire(EffectType::Explode(ExplodeConfig {
                            range:  OrderedFloat(48.0),
                            damage: OrderedFloat(1.0),
                        }))),
                    ),
                ),
                RootNode::Stamp(
                    StampTarget::Bolt,
                    Tree::When(
                        Trigger::Bumped,
                        Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                            multiplier: OrderedFloat(1.2),
                        }))),
                    ),
                ),
            ]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app
        .world_mut()
        .spawn((Cell, CellTypeAlias("M".to_owned())))
        .id();
    let bolt_entity = app.world_mut().spawn(Bolt).id();
    app.update();

    // Cell gets the Cell-targeted effect
    let cell_bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should have BoundEffects for Cell-targeted effect");
    assert_eq!(
        cell_bound.0.len(),
        1,
        "cell should have 1 BoundEffects entry (Cell-targeted only)"
    );

    // Bolt gets the Bolt-targeted effect
    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt_entity)
        .expect("bolt should have BoundEffects for Bolt-targeted effect");
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "bolt should have 1 BoundEffects entry (Bolt-targeted only)"
    );
}
