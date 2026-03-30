//! Desugaring tests for `dispatch_chip_effects` — behaviors 1-13.
//!
//! Tests that non-Breaker `RootEffect::On` targets are desugared to
//! `When(NodeStart, On(<original_target>, ...))` and dispatched to the Breaker
//! entity's `BoundEffects`.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{
        BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, StagedEffects, Target,
        Trigger,
    },
};

// ── Section A: AllCells desugaring ──

// ── Behavior 1: AllCells wraps in When(NodeStart, On(AllCells, ...)) ──

#[test]
fn all_cells_target_desugars_to_when_node_start_on_all_cells() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Fortify".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    // Only Breaker exists — no Cells (simulating ChipSelect state)
    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Fortify");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 BoundEffects entry from desugared AllCells"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Cell Fortify", "chip_name must be preserved");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::Shield { stacks: 1 })
                    )
                )
            )
        ),
        "Expected When(NodeStart, [On(AllCells, permanent: true, [When(Impacted(Bolt), [Do(Shield(1))])])]), got {node:?}"
    );
}

// ── Behavior 1 edge case: AllCells with multiple children ──

#[test]
fn all_cells_with_multiple_children_all_wrapped_in_single_on_node() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Multi Cell".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
                },
                EffectNode::When {
                    trigger: Trigger::Died,
                    then: vec![EffectNode::Do(EffectKind::Explode {
                        range: 40.0,
                        damage_mult: 1.0,
                    })],
                },
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Multi Cell");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared entry (not 2)"
    );

    let (_, node) = &bound.0[0];
    // Verify the inner On node's `then` has 2 children
    if let EffectNode::When {
        trigger: Trigger::NodeStart,
        then: outer,
    } = node
    {
        assert_eq!(outer.len(), 1, "Should have 1 On child");
        if let EffectNode::On {
            target: Target::AllCells,
            permanent: true,
            then: inner,
        } = &outer[0]
        {
            assert_eq!(
                inner.len(),
                2,
                "Inner On node should have 2 children (both When nodes)"
            );
        } else {
            panic!(
                "Expected On(AllCells, permanent: true, ...), got {:?}",
                outer[0]
            );
        }
    } else {
        panic!("Expected When(NodeStart, ...), got {node:?}");
    }
}

// ── Behavior 2: AllCells desugaring does not dispatch to Cells ──

#[test]
fn all_cells_desugaring_does_not_create_bound_effects_on_cells() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Fortify",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No cells spawned — simulating ChipSelect state
    select_chip(&mut app, "Cell Fortify");

    app.update();

    // Only Breaker should have BoundEffects modified
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 desugared BoundEffects entry"
    );
}

// ── Behavior 2 edge case: Existing BoundEffects preserved ──

#[test]
fn desugaring_preserves_existing_bound_effects_entries() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Fortify",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // Spawn breaker with 2 existing BoundEffects entries
    let breaker = {
        use crate::{
            breaker::components::Breaker,
            effect::effects::{bump_force::ActiveBumpForces, size_boost::ActiveSizeBoosts},
        };

        let existing = BoundEffects(vec![
            (
                "OldChip1".to_owned(),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                },
            ),
            (
                "OldChip2".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                },
            ),
        ]);

        app.world_mut()
            .spawn((
                Breaker,
                existing,
                StagedEffects::default(),
                ActiveBumpForces::default(),
                ActiveSizeBoosts::default(),
            ))
            .id()
    };

    select_chip(&mut app, "Cell Fortify");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        3,
        "Should have 2 old + 1 new desugared = 3 entries"
    );
    assert_eq!(bound.0[0].0, "OldChip1", "first old entry preserved");
    assert_eq!(bound.0[1].0, "OldChip2", "second old entry preserved");
    assert_eq!(bound.0[2].0, "Cell Fortify", "new desugared entry appended");
}

// ── Section B: AllWalls desugaring ──

// ── Behavior 3: AllWalls wraps in When(NodeStart, On(AllWalls, ...)) ──

#[test]
fn all_walls_target_desugars_to_when_node_start_on_all_walls() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Boost",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No walls spawned
    select_chip(&mut app, "Wall Boost");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 desugared entry for AllWalls"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Wall Boost");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllWalls,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        ..
                    }
                )
            )
        ),
        "Expected When(NodeStart, [On(AllWalls, permanent: true, ...)]), got {node:?}"
    );
}

// ── Behavior 3 edge case: Zero Breakers — no panic ──

#[test]
fn all_walls_with_zero_breakers_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Boost",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No breaker, no walls
    select_chip(&mut app, "Wall Boost");

    // Should not panic
    app.update();
}

// ── Section C: AllBolts desugaring ──

// ── Behavior 4: AllBolts wraps in When(NodeStart, On(AllBolts, ...)) ──

#[test]
fn all_bolts_target_desugars_to_when_node_start_on_all_bolts() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bolt Chain",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Do(EffectKind::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 500.0,
            })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No bolts spawned
    select_chip(&mut app, "Bolt Chain");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have exactly 1 desugared entry for AllBolts"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Bolt Chain");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllBolts,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        ..
                    }
                )
            )
        ),
        "Expected When(NodeStart, [On(AllBolts, permanent: true, ...)]), got {node:?}"
    );
}

// ── Behavior 4 edge case: AllBolts with bare Do — Do is deferred, not fired ──

#[test]
fn all_bolts_with_bare_do_child_deferred_not_fired_immediately() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Bolt Damage".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllBolts,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.3))],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No bolts exist — simulate ChipSelect
    select_chip(&mut app, "Bolt Damage");

    app.update();

    // The bare Do should NOT fire immediately — it should be wrapped in desugaring
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared entry (bare Do wrapped in When(NodeStart, On(...)))"
    );

    // Verify the Do is inside the desugared tree
    let (_, node) = &bound.0[0];
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllBolts,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::Do(EffectKind::DamageBoost(v)) if (*v - 1.3).abs() < f32::EPSILON
                )
            )
        ),
        "Bare Do should be wrapped in When(NodeStart, [On(AllBolts, [Do(DamageBoost(1.3))])]), got {node:?}"
    );
}

// ── Section D: Singular target desugaring ──

// ── Behavior 5: Bolt target desugars ──

#[test]
fn bolt_target_desugars_to_when_node_start_on_bolt() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Bolt Speed".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Speed");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Breaker should have 1 desugared entry for Bolt target"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Bolt Speed");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::Bolt,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::Do(EffectKind::SpeedBoost { multiplier }) if (*multiplier - 1.2).abs() < f32::EPSILON
                )
            )
        ),
        "Expected When(NodeStart, [On(Bolt, permanent: true, [Do(SpeedBoost(1.2))])]), got {node:?}"
    );
}

// ── Behavior 5 edge case: Bolt with When child ──

#[test]
fn bolt_target_with_when_child_desugars_correctly() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Bolt Bump".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::Bumped,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Bump");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);
    let (_, node) = &bound.0[0];
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::Bolt,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Bumped,
                        ..
                    }
                )
            )
        ),
        "Expected When(NodeStart, [On(Bolt, [When(Bumped, ...)])]), got {node:?}"
    );
}

// ── Behavior 6: Cell target desugars ──

#[test]
fn cell_target_desugars_to_when_node_start_on_cell() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Armor",
        Target::Cell,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 2 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Armor");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Cell Armor");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::Cell,
                    permanent: true,
                    ..
                }
            )
        ),
        "Expected When(NodeStart, [On(Cell, permanent: true, ...)]), got {node:?}"
    );
}

// ── Behavior 7: Wall target desugars ──

#[test]
fn wall_target_desugars_to_when_node_start_on_wall() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Reflect",
        Target::Wall,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Reflect");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Wall Reflect");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::Wall,
                    permanent: true,
                    ..
                }
            )
        ),
        "Expected When(NodeStart, [On(Wall, permanent: true, ...)]), got {node:?}"
    );
}

// ── Behavior 7 edge case: Wall with Until child ──

#[test]
fn wall_target_with_until_child_desugars_correctly() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Wall Timed".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Wall,
            then: vec![EffectNode::Until {
                trigger: Trigger::TimeExpires(5.0),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 2.0 })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Timed");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);

    let (_, node) = &bound.0[0];
    if let EffectNode::When {
        trigger: Trigger::NodeStart,
        then: outer,
    } = node
    {
        if let EffectNode::On {
            target: Target::Wall,
            permanent: true,
            then: inner,
        } = &outer[0]
        {
            assert_eq!(inner.len(), 1);
            assert!(
                matches!(
                    &inner[0],
                    EffectNode::Until {
                        trigger: Trigger::TimeExpires(t),
                        ..
                    } if (*t - 5.0).abs() < f32::EPSILON
                ),
                "Inner child should be Until(TimeExpires(5.0), ...), got {:?}",
                inner[0]
            );
        } else {
            panic!(
                "Expected On(Wall, permanent: true, ...), got {:?}",
                outer[0]
            );
        }
    } else {
        panic!("Expected When(NodeStart, ...), got {node:?}");
    }
}

// ── Section E: Breaker target NOT desugared ──

// ── Behavior 8: Breaker dispatches directly (no desugaring) ──

#[test]
fn breaker_target_dispatches_directly_not_desugared() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Breaker Shield",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Breaker Shield");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1, "Breaker should have 1 BoundEffects entry");

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Breaker Shield");
    // The node should be the direct When(PerfectBump, ...) — NOT wrapped in When(NodeStart, ...)
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then,
            } if then.len() == 1 && matches!(&then[0], EffectNode::Do(EffectKind::Shield { stacks: 1 }))
        ),
        "Breaker target should NOT be desugared — expected When(PerfectBump, [Do(Shield(1))]), got {node:?}"
    );
}

// ── Behavior 8 edge case: Breaker bare Do fires immediately ──

#[test]
fn breaker_target_bare_do_fires_immediately() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Breaker Size".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::Do(EffectKind::SizeBoost(1.15))],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Breaker Size");

    app.update();

    // SizeBoost should have fired immediately on Breaker
    let sizes = app
        .world()
        .get::<crate::effect::effects::size_boost::ActiveSizeBoosts>(breaker)
        .unwrap();
    assert_eq!(
        sizes.0,
        vec![1.15],
        "SizeBoost(1.15) should fire immediately on Breaker (not desugared)"
    );

    // BoundEffects should be empty (bare Do fires, not pushed)
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should remain empty for bare Do on Breaker target"
    );
}

// ── Section F: Nested On NOT desugared ──

// ── Behavior 9: Nested On(Bolt) inside Breaker root is not desugared ──

#[test]
fn nested_on_bolt_inside_breaker_root_not_desugared() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Nested Bolt".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                }],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let _breaker = spawn_breaker(&mut app);
    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Nested Bolt");

    app.update();

    // The nested On(Bolt) should resolve against the existing Bolt entity
    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "Bolt should have 1 BoundEffects entry from nested On resolution"
    );
    assert_eq!(bolt_bound.0[0].0, "Nested Bolt");
    assert!(
        matches!(
            &bolt_bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Bumped,
                ..
            }
        ),
        "Bolt's entry should be the inner When(Bumped, ...), not a desugared When(NodeStart, ...)"
    );
}

// ── Behavior 9 edge case: Nested On(AllCells) inside Breaker root with Cells present ──

#[test]
fn nested_on_all_cells_inside_breaker_root_resolves_directly() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Nested Cells".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::On {
                target: Target::AllCells,
                permanent: false,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let _breaker = spawn_breaker(&mut app);
    let cell = spawn_cell(&mut app);
    select_chip(&mut app, "Nested Cells");

    app.update();

    // Nested On(AllCells) resolves directly (not desugared) because it's inside a Breaker root
    let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        cell_bound.0.len(),
        1,
        "Cell should have 1 BoundEffects entry from nested On(AllCells) direct resolution"
    );
}

// ── Section G: Missing Breaker during desugaring ──

// ── Behavior 10: No Breaker entity — silently dropped ──

#[test]
fn no_breaker_entity_desugaring_silently_dropped() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Fortify",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No Breaker spawned, no Cells spawned
    select_chip(&mut app, "Cell Fortify");

    // Should not panic
    app.update();

    // Chip should still be added to inventory
    let inventory = app
        .world()
        .resource::<crate::chips::inventory::ChipInventory>();
    assert_eq!(
        inventory.stacks("Cell Fortify"),
        1,
        "Chip should be in inventory even though no Breaker exists for dispatch"
    );
}

// ── Section H: Multiple RootEffects with mixed targets ──

// ── Behavior 11: Breaker + AllBolts — Breaker direct, AllBolts desugars ──

#[test]
fn mixed_breaker_and_all_bolts_targets_dispatched_correctly() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Parry Multi".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            },
            RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 500.0,
                    })],
                }],
            },
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    // No bolts spawned — AllBolts should desugar
    select_chip(&mut app, "Parry Multi");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Breaker should have 2 entries: 1 direct (Breaker target) + 1 desugared (AllBolts target)"
    );

    // Entry 0: Direct Breaker dispatch
    let (name0, node0) = &bound.0[0];
    assert_eq!(name0, "Parry Multi");
    assert!(
        matches!(
            node0,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ),
        "First entry should be direct When(PerfectBump, ...), got {node0:?}"
    );

    // Entry 1: Desugared AllBolts
    let (name1, node1) = &bound.0[1];
    assert_eq!(name1, "Parry Multi");
    assert!(
        matches!(
            node1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllBolts,
                    permanent: true,
                    ..
                }
            )
        ),
        "Second entry should be desugared When(NodeStart, [On(AllBolts, ...)]), got {node1:?}"
    );
}

// ── Behavior 11 edge case: Three RootEffects (Breaker + AllCells + AllWalls) ──

#[test]
fn three_root_effects_breaker_direct_cells_and_walls_desugared() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Triple".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
                }],
            },
            RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
            RootEffect::On {
                target: Target::AllWalls,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Triple");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        3,
        "Breaker should have 3 entries: 1 direct + 2 desugared"
    );

    // First entry: direct Breaker dispatch
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ),
        "Entry 0 should be direct When(PerfectBump, ...)"
    );

    // Second and third: desugared
    assert!(
        matches!(
            &bound.0[1].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Entry 1 should be desugared When(NodeStart, ...)"
    );
    assert!(
        matches!(
            &bound.0[2].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Entry 2 should be desugared When(NodeStart, ...)"
    );
}

// ── Section I: permanent: true in desugared On node ──

// ── Behavior 12: Desugared inner On has permanent: true ──

#[test]
fn desugared_inner_on_node_has_permanent_true() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bolt Buff",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::Bumped,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Buff");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);

    let (_, node) = &bound.0[0];
    if let EffectNode::When {
        trigger: Trigger::NodeStart,
        then: outer,
    } = node
    {
        assert_eq!(outer.len(), 1);
        match &outer[0] {
            EffectNode::On {
                target: Target::AllBolts,
                permanent,
                ..
            } => {
                assert!(
                    *permanent,
                    "Inner On node must have permanent: true, got permanent: false"
                );
            }
            other => panic!("Expected On(AllBolts, ...), got {other:?}"),
        }
    } else {
        panic!("Expected When(NodeStart, ...), got {node:?}");
    }
}

// ── Section J: chip_name preserved through desugaring ──

// ── Behavior 13: Desugared effects preserve chip display name ──

#[test]
fn desugared_effects_preserve_chip_display_name() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Blazing Cell Armor",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Blazing Cell Armor");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(
        bound.0[0].0, "Blazing Cell Armor",
        "chip_name must be exactly 'Blazing Cell Armor' (not lowercased or modified)"
    );
}

// ── Behavior 13 edge case: Empty string chip name ──

#[test]
fn desugared_effects_preserve_empty_string_chip_name() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: String::new(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(
        bound.0[0].0, "",
        "chip_name should be empty string when chip has empty name"
    );
}

// ── Section K: End-to-end desugaring → NodeStart → Cell resolution ──

/// System that evaluates `NodeStart` trigger on all entities with `BoundEffects`.
/// Mirrors `bridge_node_start` from `effect::triggers::node_start` (which is
/// module-private), using the public(crate) evaluate helpers.
fn sys_evaluate_node_start(
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    use crate::effect::triggers::evaluate::{evaluate_bound_effects, evaluate_staged_effects};

    for (entity, bound, mut staged) in &mut query {
        evaluate_bound_effects(
            &Trigger::NodeStart,
            entity,
            bound,
            &mut staged,
            &mut commands,
        );
        evaluate_staged_effects(&Trigger::NodeStart, entity, &mut staged, &mut commands);
    }
}

/// Asserts that the given cell entity has exactly one `BoundEffects` entry
/// matching `When(Impacted(Bolt), [Do(Shield(1))])` with chip name "Cell Shield".
fn assert_cell_has_shield_bound_effect(app: &App, cell: Entity, label: &str) {
    let bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "After NodeStart, {label} should have 1 BoundEffects entry, got {}",
        bound.0.len()
    );

    let (chip, node) = &bound.0[0];
    assert_eq!(
        chip, "Cell Shield",
        "{label}'s BoundEffects chip_name should be 'Cell Shield'"
    );
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: do_children,
            } if do_children.len() == 1 && matches!(
                &do_children[0],
                EffectNode::Do(EffectKind::Shield { stacks: 1 })
            )
        ),
        "{label} should have When(Impacted(Bolt), [Do(Shield(1))]), got {node:?}"
    );
}

/// Setup helper for the E2E desugaring test.
///
/// Builds a test app with dispatch (but NOT `NodeStart` evaluation), inserts a
/// "Cell Shield" chip definition targeting `AllCells`, spawns a Breaker and
/// two Cells, selects the chip, and runs one update (Phase 1: dispatch only).
///
/// The caller is responsible for registering `sys_evaluate_node_start` after
/// verifying Phase 1 assertions (desugared entry on Breaker, 0 `BoundEffects` on
/// Cells).
///
/// Returns `(app, breaker, cell_a, cell_b)`.
fn setup_e2e_desugaring_app() -> (App, Entity, Entity, Entity) {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Cell Shield".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::AllCells,
            then: vec![EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Bolt),
                then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
            }],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);

    select_chip(&mut app, "Cell Shield");
    app.update();

    // Precondition: cells have no BoundEffects before NodeStart fires.
    for cell in [cell_a, cell_b] {
        let bound = app.world().get::<BoundEffects>(cell).unwrap();
        assert!(
            bound.0.is_empty(),
            "Cell should have 0 BoundEffects entries before NodeStart"
        );
    }

    (app, breaker, cell_a, cell_b)
}

/// End-to-end integration test: chip selection -> desugaring -> `NodeStart` trigger
/// -> cells get permanent `BoundEffects`.
///
/// Verifies the full chain: `dispatch_chip_effects` desugars `AllCells` target
/// to `When(NodeStart, On(AllCells, permanent: true, ...))` on the Breaker,
/// then when `NodeStart` fires the `On(AllCells)` node resolves to each Cell
/// entity and installs `When(Impacted(Bolt), Do(Shield { stacks: 1 }))` in
/// their `BoundEffects` (permanent, not `StagedEffects`).
#[test]
fn chip_all_cells_target_desugars_and_resolves_to_cell_bound_effects_on_node_start() {
    let (mut app, breaker, cell_a, cell_b) = setup_e2e_desugaring_app();

    // ── Phase 2 assertions: Breaker has desugared When(NodeStart) entry ──

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "After dispatch, Breaker should have exactly 1 BoundEffects entry (desugared AllCells)"
    );

    let (chip_name, node) = &breaker_bound.0[0];
    assert_eq!(chip_name, "Cell Shield", "chip_name must be 'Cell Shield'");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                then: outer,
            } if outer.len() == 1 && matches!(
                &outer[0],
                EffectNode::On {
                    target: Target::AllCells,
                    permanent: true,
                    then: inner,
                } if inner.len() == 1 && matches!(
                    &inner[0],
                    EffectNode::When {
                        trigger: Trigger::Impacted(ImpactTarget::Bolt),
                        then: do_children,
                    } if do_children.len() == 1 && matches!(
                        &do_children[0],
                        EffectNode::Do(EffectKind::Shield { stacks: 1 })
                    )
                )
            )
        ),
        "Breaker's entry should be When(NodeStart, [On(AllCells, permanent: true, \
         [When(Impacted(Bolt), [Do(Shield(1))])])]), got {node:?}"
    );

    // ── Phase 3: Register evaluate system, fire NodeStart trigger ──

    // Now that Phase 2 is verified, add the evaluate system so NodeStart
    // processing happens on the next update (not in the same frame as
    // dispatch).
    app.add_systems(
        Update,
        sys_evaluate_node_start
            .after(crate::chips::systems::dispatch_chip_effects::dispatch_chip_effects),
    );

    // Clear pending selections so dispatch_chip_effects does not re-process
    // on the second update.
    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .clear();

    // Run another update — sys_evaluate_node_start evaluates NodeStart on
    // all entities with BoundEffects.  The Breaker's When(NodeStart) fires,
    // pushing On(AllCells) to StagedEffects, which then resolves to Cell
    // entities via ResolveOnCommand.
    app.update();

    // ── Phase 3 assertions: Both Cell entities have permanent BoundEffects ──

    assert_cell_has_shield_bound_effect(&app, cell_a, "Cell A");
    assert_cell_has_shield_bound_effect(&app, cell_b, "Cell B");

    // ── Phase 3 assertions: Both Cell entities have 0 StagedEffects ──
    // permanent: true means children go to BoundEffects, not StagedEffects.

    let first_staged = app.world().get::<StagedEffects>(cell_a).unwrap();
    assert!(
        first_staged.0.is_empty(),
        "Cell A should have 0 StagedEffects (permanent routing), got {}",
        first_staged.0.len()
    );

    let second_staged = app.world().get::<StagedEffects>(cell_b).unwrap();
    assert!(
        second_staged.0.is_empty(),
        "Cell B should have 0 StagedEffects (permanent routing), got {}",
        second_staged.0.len()
    );

    // ── Phase 3 assertions: Breaker's When(NodeStart) processed ──
    // After NodeStart evaluation, the On(AllCells) child was pushed to
    // StagedEffects and consumed by ResolveOnCommand.  The Breaker's
    // StagedEffects should be empty (On node consumed).

    let breaker_staged = app.world().get::<StagedEffects>(breaker).unwrap();
    assert!(
        breaker_staged.0.is_empty(),
        "Breaker's StagedEffects should be empty after On(AllCells) was consumed, got {} entries",
        breaker_staged.0.len()
    );
}
