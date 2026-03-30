//! Breaker-targeted push-to-BoundEffects tests — behaviors 2, 3, 4, and 6.
//!
//! These tests verify that `When`, `Until`, and `Once` children targeting
//! Breaker push their effect trees to `BoundEffects` (not fired immediately).

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, Target, Trigger},
};

// ── Behavior 2: `When` child targeting Breaker pushes to BoundEffects ──

#[test]
fn when_child_targeting_breaker_pushes_to_bound_effects() {
    let mut app = test_app();

    let shockwave = EffectKind::Shockwave {
        base_range: 20.0,
        range_per_level: 5.0,
        stacks: 1,
        speed: 400.0,
    };
    let def = ChipDefinition::test_on(
        "Minor Cascade",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(shockwave)],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Minor Cascade");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
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
            target: Target::Breaker,
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

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Dual When");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries for the two When nodes"
    );
}

// ── Behavior 3: `Until` child pushes full tree to BoundEffects ──

#[test]
fn until_child_targeting_breaker_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Basic Overclock",
        Target::Breaker,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Until {
                trigger: Trigger::TimeExpires(2.0),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Basic Overclock");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry for the When {{ PerfectBump, [Until(...)] }} tree"
    );

    let (chip_name, node) = &bound.0[0];
    assert_eq!(chip_name, "Basic Overclock");
    assert!(
        matches!(node, EffectNode::When { trigger: Trigger::PerfectBump, then } if then.len() == 1),
        "should be When {{ PerfectBump, [Until(...)] }}, got {node:?}"
    );
}

// ── Behavior 3 edge case: Bare `Until` at `On` top-level pushes to BoundEffects ──

#[test]
fn bare_until_at_top_level_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bare Until",
        Target::Breaker,
        EffectNode::Until {
            trigger: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bare Until");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
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
fn once_child_targeting_breaker_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Test Once",
        Target::Breaker,
        EffectNode::Once(vec![EffectNode::Do(EffectKind::DamageBoost(2.5))]),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Test Once");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
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

    let def = ChipDefinition::test_on(
        "Once When",
        Target::Breaker,
        EffectNode::Once(vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }]),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Once When");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Once wrapping When should be pushed to BoundEffects, not fired"
    );
}

// ── Behavior 6: `When` child targeting Breaker pushes to BoundEffects ──

#[test]
fn when_child_targeting_breaker_pushes_to_bound_effects_with_shield() {
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
