//! Breaker-targeted bare `Do` dispatch tests — behaviors 1 and 5.
//!
//! These tests verify that bare `Do` children targeting Breaker fire their
//! effects immediately (not pushed to `BoundEffects`).

use bevy::prelude::*;

use super::super::helpers::*;
use crate::{
    chips::definition::ChipDefinition,
    effect::{
        BoundEffects, EffectKind, EffectNode, RootEffect, Target,
        effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
    },
};

// ── Behavior 1: Bare `Do` child targeting Breaker fires immediately ──

#[test]
fn bare_do_targeting_breaker_fires_damage_boost_immediately() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Minor Damage Boost",
        Target::Breaker,
        EffectNode::Do(EffectKind::DamageBoost(1.1)),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Minor Damage Boost");

    app.update();

    let boosts = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        boosts.0,
        vec![1.1],
        "DamageBoost(1.1) should have been fired immediately on breaker"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
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
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 }),
                EffectNode::Do(EffectKind::DamageBoost(1.05)),
            ],
        }],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Multi Do");

    app.update();

    let speed = app.world().get::<ActiveSpeedBoosts>(breaker).unwrap();
    assert_eq!(
        speed.0,
        vec![1.2],
        "SpeedBoost(1.2) should have been fired immediately"
    );

    let damage = app.world().get::<ActiveDamageBoosts>(breaker).unwrap();
    assert_eq!(
        damage.0,
        vec![1.05],
        "DamageBoost(1.05) should have been fired immediately"
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
