//! Breaker target dispatch tests — behavior 8.
//!
//! Breaker targets dispatch directly — Fire effects fire immediately,
//! other trees stamp to `BoundEffects`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{SizeBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, StampTarget, Tree, Trigger},
    },
};

#[test]
fn breaker_fire_dispatches_immediately() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Speed Up",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.3),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Speed Up");

    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "SpeedBoost should fire immediately");

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert!(
        bound.0.is_empty(),
        "BoundEffects should be empty for bare Fire"
    );
}

#[test]
fn breaker_size_boost_fires_immediately() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Size Up",
        StampTarget::Breaker,
        Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
            multiplier: OrderedFloat(1.2),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Size Up");

    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "SizeBoost should fire immediately");
}

#[test]
fn breaker_when_stamps_to_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Triggered",
        StampTarget::Breaker,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Triggered");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1, "When tree should stamp to BoundEffects");
    assert!(
        matches!(&bound.0[0].1, Tree::When(Trigger::PerfectBumped, _)),
        "Should be When(PerfectBumped, ...), got {:?}",
        bound.0[0].1
    );
}
