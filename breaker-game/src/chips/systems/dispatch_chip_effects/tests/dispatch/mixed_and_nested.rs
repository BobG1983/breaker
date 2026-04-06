//! Mixed dispatch and nested `On` tests — behaviors 13-14 and nested dispatch.
//!
//! Tests for chips with mixed `Do`+`When` children, multiple `RootEffect::On`
//! entries, and nested `EffectNode::On` dispatching to inner targets.

use bevy::prelude::*;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect::{
        BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger,
        effects::damage_boost::ActiveDamageBoosts,
    },
};

// ── Behavior 13: Mixed `Do` and `When` on Breaker — `Do` fires, `When` pushes ──

#[test]
fn mixed_do_and_when_do_fires_when_pushes() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Mixed".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Breaker,
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

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Mixed");

    app.update();

    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.2],
        "DamageBoost(1.2) should have been fired immediately"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
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
            target: Target::Breaker,
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

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Interleaved");

    app.update();

    // The bare Do should fire immediately
    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.1],
        "DamageBoost(1.1) should have been fired immediately"
    );

    // Both When nodes should be pushed to BoundEffects
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
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
                    then: vec![EffectNode::Do(EffectKind::Shield {
                        duration: 5.0,
                        reflection_cost: 0.0,
                    })],
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
    select_chip(&mut app, "Parry Multi");

    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        2,
        "Breaker should have 2 BoundEffects entries: 1 direct (Breaker target) + 1 desugared (AllBolts)"
    );
    assert_eq!(breaker_bound.0[0].0, "Parry Multi");
    // Entry 0: direct Breaker dispatch
    assert!(
        matches!(
            &breaker_bound.0[0].1,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ),
        "First entry should be direct When(PerfectBump, ...)"
    );
    // Entry 1: desugared AllBolts
    assert_eq!(breaker_bound.0[1].0, "Parry Multi");
    assert!(
        matches!(
            &breaker_bound.0[1].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Second entry should be desugared When(NodeStart, [On(AllBolts, ...)])"
    );
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
                    then: vec![EffectNode::Do(EffectKind::Shield {
                        duration: 5.0,
                        reflection_cost: 0.0,
                    })],
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
    select_chip(&mut app, "Triple");

    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        3,
        "Breaker should have 3 BoundEffects entries: 1 direct + 2 desugared"
    );

    // First: direct Breaker dispatch
    assert!(
        matches!(
            &breaker_bound.0[0].1,
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
            &breaker_bound.0[1].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Entry 1 should be desugared When(NodeStart, [On(Bolt, ...)])"
    );
    assert!(
        matches!(
            &breaker_bound.0[2].1,
            EffectNode::When {
                trigger: Trigger::NodeStart,
                ..
            }
        ),
        "Entry 2 should be desugared When(NodeStart, [On(AllCells, ...)])"
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
