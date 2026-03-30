use bevy::prelude::*;

use super::helpers::{make_cell_def, test_app};
use crate::{
    bolt::components::Bolt,
    cells::components::{Cell, CellEffectsDispatched, CellTypeAlias},
    effect::{BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
};

// ── Behavior 8: Do children are fired immediately, not stored in BoundEffects ──

#[test]
fn do_children_are_not_stored_in_bound_effects() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'D',
        make_cell_def(
            "immediate_effect_cell",
            'D',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Cell,
                then: vec![
                    EffectNode::Do(EffectKind::DamageBoost(1.5)),
                    EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    },
                ],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('D'))).id();
    app.update();

    // BoundEffects should have exactly 1 entry (the When node), NOT the Do node
    let bound = app
        .world()
        .get::<BoundEffects>(cell_entity)
        .expect("cell should have BoundEffects");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry (When), not the Do child; got {}",
        bound.0.len()
    );
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Died,
                ..
            }
        ),
        "the single BoundEffects entry should be the When {{ Died }} node, got {:?}",
        bound.0[0].1
    );
}

// ── Behavior 8 edge case: All children are Do nodes ──

#[test]
fn all_do_children_results_in_no_bound_effects_entries() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'D',
        make_cell_def(
            "all_do_cell",
            'D',
            10.0,
            Some(vec![RootEffect::On {
                target: Target::Cell,
                then: vec![
                    EffectNode::Do(EffectKind::DamageBoost(1.5)),
                    EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 }),
                ],
            }]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('D'))).id();
    app.update();

    // If BoundEffects was inserted, it should be empty (all children were Do nodes)
    // Or BoundEffects might not be inserted at all -- both are acceptable
    if let Some(bound) = app.world().get::<BoundEffects>(cell_entity) {
        assert_eq!(
            bound.0.len(),
            0,
            "BoundEffects should be empty when all children are Do nodes, got {}",
            bound.0.len()
        );
    }

    // Cell should still have marker since it had effects
    assert!(
        app.world()
            .get::<CellEffectsDispatched>(cell_entity)
            .is_some(),
        "cell should have CellEffectsDispatched even when all children are Do nodes"
    );
}

// ── Behavior 9: Cell with multiple RootEffects gets all dispatched ──

#[test]
fn cell_with_multiple_root_effects_gets_all_dispatched() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'M',
        make_cell_def(
            "multi_effect_cell",
            'M',
            20.0,
            Some(vec![
                RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                },
                RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
                    }],
                },
            ]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('M'))).id();
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
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Died,
                ..
            }
        ),
        "first entry should be When {{ Died }}, got {:?}",
        bound.0[0].1
    );

    // Second: Impacted(Bolt)->DamageBoost
    assert!(
        matches!(
            &bound.0[1].1,
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                ..
            }
        ),
        "second entry should be When {{ Impacted(Bolt) }}, got {:?}",
        bound.0[1].1
    );
}

// ── Behavior 9 edge case: Mix of Target::Cell and Target::Bolt ──

#[test]
fn cell_with_mixed_targets_dispatches_to_correct_entities() {
    let mut registry = crate::cells::resources::CellTypeRegistry::default();
    registry.insert(
        'M',
        make_cell_def(
            "mixed_target_cell",
            'M',
            10.0,
            Some(vec![
                RootEffect::On {
                    target: Target::Cell,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Died,
                        then: vec![EffectNode::Do(EffectKind::Explode {
                            range: 48.0,
                            damage_mult: 1.0,
                        })],
                    }],
                },
                RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Bumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                    }],
                },
            ]),
        ),
    );

    let mut app = test_app(registry);
    let cell_entity = app.world_mut().spawn((Cell, CellTypeAlias('M'))).id();
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
