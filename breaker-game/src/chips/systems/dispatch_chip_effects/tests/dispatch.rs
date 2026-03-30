//! Core dispatch tests for `dispatch_chip_effects` — behaviors 1-14.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{
        BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger,
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    },
};

// ── Behavior 1: Bare `Do` child targeting Bolt fires immediately ──

#[test]
fn bare_do_targeting_bolt_fires_damage_boost_immediately() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Minor Damage Boost",
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Minor Damage Boost");

    app.update();

    let boosts = app.world().get::<ActiveDamageBoosts>(bolt).unwrap();
    assert_eq!(
        boosts.0,
        vec![1.1],
        "DamageBoost(1.1) should have been fired immediately on bolt"
    );

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should remain empty for bare Do children"
    );
}

// ── Behavior 1 edge case: Multiple bare `Do` children in same `On` ──

#[test]
fn multiple_bare_do_children_all_fire_immediately() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Multi Do".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 }),
                EffectNode::Do(EffectKind::DamageBoost(1.05)),
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Multi Do");

    app.update();

    let speed = app.world().get::<ActiveSpeedBoosts>(bolt).unwrap();
    assert_eq!(
        speed.0,
        vec![1.2],
        "SpeedBoost(1.2) should have been fired immediately"
    );

    let damage = app.world().get::<ActiveDamageBoosts>(bolt).unwrap();
    assert_eq!(
        damage.0,
        vec![1.05],
        "DamageBoost(1.05) should have been fired immediately"
    );
}

// ── Behavior 2: `When` child targeting Bolt pushes to BoundEffects ──

#[test]
fn when_child_targeting_bolt_pushes_to_bound_effects() {
    let mut app = test_app();

    let shockwave = EffectKind::Shockwave {
        base_range: 20.0,
        range_per_level: 5.0,
        stacks: 1,
        speed: 400.0,
    };
    let def = ChipDefinition::test(
        "Minor Cascade",
        EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(shockwave)],
        },
        5,
    );
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Minor Cascade");

    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry for the When node"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Minor Cascade", "chip_name should match");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::CellDestroyed,
                then,
            } if then.len() == 1 && matches!(&then[0], EffectNode::Do(EffectKind::Shockwave { base_range, .. }) if (*base_range - 20.0).abs() < f32::EPSILON)
        ),
        "should be When {{ CellDestroyed, [Do(Shockwave)] }}, got {node:?}"
    );
}

// ── Behavior 2 edge case: Two `When` children in same `On` ──

#[test]
fn two_when_children_both_pushed_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Dual When".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
                },
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.2))],
                },
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Dual When");

    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries for the two When nodes"
    );
}

// ── Behavior 3: `Until` child pushes full tree to BoundEffects ──

#[test]
fn until_child_targeting_bolt_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Basic Overclock",
        EffectNode::When {
            trigger: Trigger::PerfectBumped,
            then: vec![EffectNode::Until {
                trigger: Trigger::TimeExpires(2.0),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
        },
        5,
    );
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Basic Overclock");

    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry for the When {{ PerfectBumped, [Until(...)] }} tree"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Basic Overclock");
    assert!(
        matches!(node, EffectNode::When { trigger: Trigger::PerfectBumped, then } if then.len() == 1),
        "should be When {{ PerfectBumped, [Until(...)] }}, got {node:?}"
    );
}

// ── Behavior 3 edge case: Bare `Until` at `On` top-level pushes to BoundEffects ──

#[test]
fn bare_until_at_top_level_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Bare Until",
        EffectNode::Until {
            trigger: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Bare Until");

    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Bare Until at top level should be pushed to BoundEffects"
    );
    assert!(
        matches!(&bound.0[0].1, EffectNode::Until { trigger: Trigger::TimeExpires(t), .. } if (*t - 3.0).abs() < f32::EPSILON),
        "should be Until {{ TimeExpires(3.0), ... }}"
    );
}

// ── Behavior 4: `Once` child pushes to BoundEffects ──

#[test]
fn once_child_targeting_bolt_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Test Once",
        EffectNode::Once(vec![EffectNode::Do(EffectKind::DamageBoost(2.5))]),
        5,
    );
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Test Once");

    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry for the Once node"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Test Once");
    assert!(
        matches!(node, EffectNode::Once(children) if children.len() == 1 && matches!(&children[0], EffectNode::Do(EffectKind::DamageBoost(v)) if (*v - 2.5).abs() < f32::EPSILON)),
        "should be Once([Do(DamageBoost(2.5))]), got {node:?}"
    );
}

// ── Behavior 4 edge case: `Once` wrapping a `When` node still pushed ──

#[test]
fn once_wrapping_when_still_pushed_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Once When",
        EffectNode::Once(vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }]),
        5,
    );
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Once When");

    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Once wrapping When should be pushed to BoundEffects, not fired"
    );
}

// ── Behavior 5: Bare `Do` targeting Breaker fires immediately ──

#[test]
fn bare_do_targeting_breaker_fires_size_and_bump_force() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Basic Augment".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::SizeBoost(1.15)),
                EffectNode::Do(EffectKind::BumpForce(1.15)),
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Basic Augment");

    app.update();

    let sizes = app
        .world()
        .get::<crate::effect::effects::size_boost::ActiveSizeBoosts>(breaker)
        .unwrap();
    assert_eq!(
        sizes.0,
        vec![1.15],
        "SizeBoost(1.15) should have been fired on breaker"
    );

    let forces = app
        .world()
        .get::<crate::effect::effects::bump_force::ActiveBumpForces>(breaker)
        .unwrap();
    assert_eq!(
        forces.0,
        vec![1.15],
        "BumpForce(1.15) should have been fired on breaker"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects on breaker should remain empty for bare Do children"
    );
}

// ── Behavior 6: `When` child targeting Breaker pushes to BoundEffects ──

#[test]
fn when_child_targeting_breaker_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Parry",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // Use spawn_breaker_bare (no BoundEffects) — system must insert it
    let breaker = spawn_breaker_bare(&mut app);
    select_chip(&mut app, "Parry");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects on breaker should have 1 entry for the When node"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Parry");
    assert!(
        matches!(
            node,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then,
            } if then.len() == 1 && matches!(&then[0], EffectNode::Do(EffectKind::Shield { stacks: 1 }))
        ),
        "should be When {{ PerfectBump, [Do(Shield {{ stacks: 1 }})] }}, got {node:?}"
    );
}

// ── Behavior 7: Target `AllBolts` dispatches to every Bolt entity ──

#[test]
fn all_bolts_target_dispatches_to_every_bolt() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Parry Shockwave",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
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

    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    select_chip(&mut app, "Parry Shockwave");

    app.update();

    for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
        let bound = app.world().get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
        assert_eq!(bound.0[0].0, "Parry Shockwave");
    }
}

// ── Behavior 7 edge case: Zero Bolt entities — no panic ──

#[test]
fn all_bolts_target_with_zero_bolts_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Target",
        Target::AllBolts,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    // No bolt entities spawned
    select_chip(&mut app, "Empty Target");

    // Should not panic
    app.update();
}

// ── Behavior 8: Target `Bolt` dispatches to all Bolt entities (same as AllBolts) ──

#[test]
fn bolt_target_dispatches_to_all_bolt_entities() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Slight Bolt Speed",
        EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 }),
        5,
    );
    insert_chip(&mut app, def);

    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    select_chip(&mut app, "Slight Bolt Speed");

    app.update();

    for (label, entity) in [("bolt_a", bolt_a), ("bolt_b", bolt_b)] {
        let speed = app.world().get::<ActiveSpeedBoosts>(entity).unwrap();
        assert_eq!(
            speed.0,
            vec![1.1],
            "{label} should have SpeedBoost(1.1) fired"
        );
    }
}

// ── Behavior 9: Target `AllCells` dispatches to every Cell entity ──

#[test]
fn all_cells_target_dispatches_to_every_cell() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield",
        Target::AllCells,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // Use spawn_cell_bare (no BoundEffects) — system must insert it
    let cell_a = spawn_cell_bare(&mut app);
    let cell_b = spawn_cell_bare(&mut app);
    select_chip(&mut app, "Cell Shield");

    app.update();

    for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b)] {
        let bound = app.world().get::<BoundEffects>(entity).unwrap();
        assert_eq!(bound.0.len(), 1, "{label} should have 1 BoundEffects entry");
    }
}

// ── Behavior 10: Target `Cell` dispatches same as `AllCells` ──

#[test]
fn cell_target_dispatches_to_all_cell_entities() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield Single",
        Target::Cell,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::Shield { stacks: 1 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);
    select_chip(&mut app, "Cell Shield Single");

    app.update();

    for (label, entity) in [("cell_a", cell_a), ("cell_b", cell_b)] {
        let bound = app.world().get::<BoundEffects>(entity).unwrap();
        assert_eq!(
            bound.0.len(),
            1,
            "{label} should have 1 BoundEffects entry (Cell target dispatches like AllCells)"
        );
    }
}

// ── Behavior 10 edge case: Zero Cell entities — no panic ──

#[test]
fn cell_target_with_zero_cells_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Cell",
        Target::Cell,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    select_chip(&mut app, "Empty Cell");
    app.update();
}

// ── Behavior 11: Target `AllWalls` dispatches to every Wall entity ──

#[test]
fn all_walls_target_dispatches_to_every_wall() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Effect",
        Target::AllWalls,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    // Use spawn_wall_bare (no BoundEffects) — system must insert it
    let wall = spawn_wall_bare(&mut app);
    select_chip(&mut app, "Wall Effect");

    app.update();

    let bound = app.world().get::<BoundEffects>(wall).unwrap();
    assert_eq!(bound.0.len(), 1, "Wall should have 1 BoundEffects entry");
}

// ── Behavior 12: Target `Wall` dispatches same as `AllWalls` ──

#[test]
fn wall_target_dispatches_to_all_wall_entities() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Single",
        Target::Wall,
        EffectNode::When {
            trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let wall = spawn_wall(&mut app);
    select_chip(&mut app, "Wall Single");

    app.update();

    let bound = app.world().get::<BoundEffects>(wall).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Wall should have 1 BoundEffects entry (Wall target dispatches like AllWalls)"
    );
}

// ── Behavior 12 edge case: Zero Wall entities — no panic ──

#[test]
fn wall_target_with_zero_walls_no_panic() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Empty Wall",
        Target::Wall,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
        5,
    );
    insert_chip(&mut app, def);

    select_chip(&mut app, "Empty Wall");
    app.update();
}

// ── Behavior 13: Mixed `Do` and `When` — `Do` fires, `When` pushes ──

#[test]
fn mixed_do_and_when_do_fires_when_pushes() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Mixed".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(1.2)),
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(EffectKind::Shockwave {
                        base_range: 24.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                },
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Mixed");

    app.update();

    let damage = app.world().get::<ActiveDamageBoosts>(bolt).unwrap();
    assert_eq!(
        damage.0,
        vec![1.2],
        "DamageBoost(1.2) should have been fired immediately"
    );

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry for the When node"
    );
    assert_eq!(bound.0[0].0, "Mixed");
}

// ── Behavior 13 edge case: Interleaved `Do` between two `When` nodes ──

#[test]
fn interleaved_do_between_two_when_nodes() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Interleaved".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![
                EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(EffectKind::Shockwave {
                        base_range: 24.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                },
                EffectNode::Do(EffectKind::DamageBoost(1.1)),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
                },
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Interleaved");

    app.update();

    // The bare Do should fire immediately
    let damage = app.world().get::<ActiveDamageBoosts>(bolt).unwrap();
    assert_eq!(
        damage.0,
        vec![1.1],
        "DamageBoost(1.1) should have been fired immediately"
    );

    // Both When nodes should be pushed to BoundEffects
    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Both When nodes should be pushed to BoundEffects"
    );
    assert_eq!(bound.0[0].0, "Interleaved");
    assert!(
        matches!(
            &bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::CellDestroyed,
                ..
            }
        ),
        "first BoundEffects entry should be When {{ CellDestroyed, ... }}"
    );
    assert_eq!(bound.0[1].0, "Interleaved");
    assert!(
        matches!(
            &bound.0[1].1,
            EffectNode::When {
                trigger: Trigger::Bump,
                ..
            }
        ),
        "second BoundEffects entry should be When {{ Bump, ... }}"
    );
}

// ── Behavior 14: Chip with multiple `RootEffect::On` entries dispatches all ──

#[test]
fn multiple_root_effects_all_dispatched() {
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
                    trigger: Trigger::PerfectBump,
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
    let bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Parry Multi");

    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry (Shield When node)"
    );
    assert_eq!(breaker_bound.0[0].0, "Parry Multi");

    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "Bolt should have 1 BoundEffects entry (Shockwave When node)"
    );
    assert_eq!(bolt_bound.0[0].0, "Parry Multi");
}

// ── Behavior 14 edge case: Three `On` entries (Breaker + Bolt + AllCells) ──

#[test]
fn three_root_effects_all_dispatched_to_appropriate_entities() {
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
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.3))],
            },
            RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    select_chip(&mut app, "Triple");

    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry"
    );

    let damage = app.world().get::<ActiveDamageBoosts>(bolt).unwrap();
    assert_eq!(
        damage.0,
        vec![1.3],
        "Bolt should have DamageBoost(1.3) fired"
    );

    let cell_bound = app.world().get::<BoundEffects>(cell).unwrap();
    assert_eq!(
        cell_bound.0.len(),
        1,
        "Cell should have 1 BoundEffects entry"
    );
}

// ── Nested `EffectNode::On` behavior ──

#[test]
fn nested_on_node_dispatches_to_inner_target() {
    let mut app = test_app();

    // Chip targets Breaker at top level, but has an inner On { Bolt } that should
    // dispatch a When node to the bolt entity.
    let def = ChipDefinition {
        name: "Nested On".to_owned(),
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
    select_chip(&mut app, "Nested On");

    app.update();

    let bolt_bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "Bolt should have 1 BoundEffects entry from nested On node"
    );
    assert_eq!(bolt_bound.0[0].0, "Nested On");
    assert!(
        matches!(
            &bolt_bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::Bumped,
                ..
            }
        ),
        "inner When node should be pushed to bolt's BoundEffects"
    );
}
