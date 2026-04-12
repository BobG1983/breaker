//! Breaker-targeted bare `Fire` dispatch tests — behaviors 1 and 5.
//!
//! These tests verify that bare `Fire` children targeting Breaker fire their
//! effects immediately via `fire_effect` (not pushed to `BoundEffects`).

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{BumpForceConfig, DamageBoostConfig, SizeBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree},
    },
};

// ── Behavior 1: Bare `Fire` child targeting Breaker fires immediately ──

#[test]
fn bare_fire_targeting_breaker_fires_damage_boost_immediately() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Minor Damage Boost",
        StampTarget::Breaker,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Minor Damage Boost");

    app.update();

    let stack = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        stack.len(),
        1,
        "DamageBoost should have been fired immediately on breaker"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should remain empty for bare Fire children"
    );
}

// ── Behavior 1 edge case: Multiple bare `Fire` children in same `Stamp` ──
// NOTE: In the new system, Stamp takes a single tree, not a list.
// Multiple effects require multiple Stamp entries in the effects vec.

#[test]
fn multiple_stamps_with_fire_all_fire_immediately() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Multi Fire".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.2),
                })),
            ),
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(1.05),
                })),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Multi Fire");

    app.update();

    let speed = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        speed.len(),
        1,
        "SpeedBoost should have been fired immediately"
    );

    let damage = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        damage.len(),
        1,
        "DamageBoost should have been fired immediately"
    );
}

// ── Behavior 5: Bare `Fire` targeting Breaker fires immediately ──

#[test]
fn bare_fire_targeting_breaker_fires_size_and_bump_force() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Basic Augment".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
                    multiplier: OrderedFloat(1.15),
                })),
            ),
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::BumpForce(BumpForceConfig {
                    multiplier: OrderedFloat(1.15),
                })),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Basic Augment");

    app.update();

    let sizes = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        sizes.len(),
        1,
        "SizeBoost should have been fired on breaker"
    );

    let forces = app
        .world()
        .get::<EffectStack<BumpForceConfig>>(breaker)
        .unwrap();
    assert_eq!(
        forces.len(),
        1,
        "BumpForce should have been fired on breaker"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects on breaker should remain empty for bare Fire children"
    );
}
