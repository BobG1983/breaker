//! Mixed target, permanent flag, `chip_name` preservation, and missing breaker
//! tests — behaviors 10-13.

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
};

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
            then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
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
                    then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
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
                    then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
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
            then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
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
                then: vec![EffectNode::Do(EffectKind::Shield { duration: 5.0 })],
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
