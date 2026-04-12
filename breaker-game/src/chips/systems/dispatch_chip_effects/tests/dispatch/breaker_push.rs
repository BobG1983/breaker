//! Breaker-targeted push-to-BoundEffects tests — behaviors 2, 3, 4, and 6.
//!
//! These tests verify that `When`, `Until`, and `Once` children targeting
//! Breaker stamp their effect trees to `BoundEffects` (not fired immediately).

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, ShockwaveConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 2: `When` child targeting Breaker pushes to BoundEffects ──

#[test]
fn when_child_targeting_breaker_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Minor Cascade",
        StampTarget::Breaker,
        Tree::When(
            Trigger::DeathOccurred(crate::effect_v3::types::EntityKind::Cell),
            Box::new(Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
                base_range: OrderedFloat(20.0),
                range_per_level: OrderedFloat(5.0),
                stacks: 1,
                speed: OrderedFloat(400.0),
            }))),
        ),
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

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Minor Cascade", "chip_name should match");
    assert!(
        matches!(tree, Tree::When(Trigger::DeathOccurred(_), _)),
        "should be When(DeathOccurred(...), ...), got {tree:?}"
    );
}

// ── Behavior 2 edge case: Two `When` trees in same chip ──

#[test]
fn two_when_stamps_both_pushed_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Dual When".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::DeathOccurred(crate::effect_v3::types::EntityKind::Cell),
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.1),
                    }))),
                ),
            ),
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.2),
                    }))),
                ),
            ),
        ],
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
        "BoundEffects should have 2 entries for the two When trees"
    );
}

// ── Behavior 3: Nested `When` with inner tree pushes full tree to BoundEffects ──

#[test]
fn when_with_nested_tree_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Basic Overclock",
        StampTarget::Breaker,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.3),
            }))),
        ),
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
        "BoundEffects should have 1 entry for the When tree"
    );

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Basic Overclock");
    assert!(
        matches!(tree, Tree::When(Trigger::PerfectBumped, _)),
        "should be When(PerfectBumped, ...), got {tree:?}"
    );
}

// ── Behavior 4: `Once` child pushes to BoundEffects ──

#[test]
fn once_child_targeting_breaker_pushes_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Test Once",
        StampTarget::Breaker,
        Tree::Once(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(2.5),
            }))),
        ),
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

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Test Once");
    assert!(
        matches!(tree, Tree::Once(Trigger::Bumped, _)),
        "should be Once(Bumped, ...), got {tree:?}"
    );
}

// ── Behavior 6: `When` child targeting Breaker stamps to BoundEffects ──

#[test]
fn when_child_targeting_breaker_stamps_to_bound_effects_with_shield() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Parry",
        StampTarget::Breaker,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration: OrderedFloat(5.0),
                reflection_cost: OrderedFloat(0.0),
            }))),
        ),
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

    let (chip_name, tree) = &bound.0[0];
    assert_eq!(chip_name, "Parry");
    assert!(
        matches!(tree, Tree::When(Trigger::PerfectBumped, _)),
        "should be When(PerfectBumped, ...), got {tree:?}"
    );
}
