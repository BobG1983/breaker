//! End-to-end dispatch tests for `ActiveWalls` target.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::assert_bound_count;
use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
};

#[test]
fn active_walls_when_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Boost",
        StampTarget::ActiveWalls,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let _wall = spawn_wall(&mut app);
    select_chip(&mut app, "Wall Boost");

    app.update();

    assert_bound_count(app.world(), breaker, 1);
}

#[test]
fn active_walls_fire_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Wall Speed",
        StampTarget::ActiveWalls,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall Speed");

    app.update();

    assert_bound_count(app.world(), breaker, 1);
}

#[test]
fn two_active_walls_chips_both_stamp() {
    let mut app = test_app();

    let def_a = ChipDefinition::test_on(
        "Wall A",
        StampTarget::ActiveWalls,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    let def_b = ChipDefinition::test_on(
        "Wall B",
        StampTarget::ActiveWalls,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        })),
        5,
    );
    insert_chip(&mut app, def_a);
    insert_chip(&mut app, def_b);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Wall A");
    select_chip(&mut app, "Wall B");

    app.update();

    assert_bound_count(app.world(), breaker, 2);
}
