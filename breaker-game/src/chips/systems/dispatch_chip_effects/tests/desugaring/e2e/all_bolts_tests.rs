//! End-to-end dispatch tests for `ActiveBolts` target.
//!
//! Verifies that `ActiveBolts` stamps correctly route to breaker's `BoundEffects`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::assert_bound_count;
use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{PiercingConfig, SpeedBoostConfig},
        types::{EffectType, StampTarget, Tree, Trigger},
    },
};

#[test]
fn active_bolts_when_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bolt Speed",
        StampTarget::ActiveBolts,
        Tree::When(
            Trigger::Bumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.2),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let _bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Bolt Speed");

    app.update();

    assert_bound_count(app.world(), breaker, 1);
}

#[test]
fn active_bolts_fire_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Bolt Pierce",
        StampTarget::ActiveBolts,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Pierce");

    app.update();

    assert_bound_count(app.world(), breaker, 1);
}

#[test]
fn two_active_bolts_chips_both_stamp() {
    let mut app = test_app();

    let def_a = ChipDefinition::test_on(
        "Bolt Speed A",
        StampTarget::ActiveBolts,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    let def_b = ChipDefinition::test_on(
        "Bolt Speed B",
        StampTarget::ActiveBolts,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        })),
        5,
    );
    insert_chip(&mut app, def_a);
    insert_chip(&mut app, def_b);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Bolt Speed A");
    select_chip(&mut app, "Bolt Speed B");

    app.update();

    assert_bound_count(app.world(), breaker, 2);
}
