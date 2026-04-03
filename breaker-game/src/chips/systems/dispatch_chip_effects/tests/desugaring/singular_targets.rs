//! Singular target desugaring tests — behaviors 5-7 (Bolt, Cell, Wall).

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, ImpactTarget, RootEffect, Target, Trigger},
};

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
            then: vec![EffectNode::Do(EffectKind::Shield {
                duration: 10.0,
                reflection_cost: 0.0,
            })],
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
