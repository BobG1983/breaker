//! End-to-end dispatch tests for `ActiveCells` target.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::assert_bound_count;
use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, SpeedBoostConfig},
        types::{EffectType, EntityKind, StampTarget, Tree, Trigger},
    },
};

#[test]
fn active_cells_when_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Shield",
        StampTarget::ActiveCells,
        Tree::When(
            Trigger::Impacted(EntityKind::Bolt),
            Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                duration: OrderedFloat(5.0),
                reflection_cost: OrderedFloat(0.0),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let _cell = spawn_cell(&mut app);
    select_chip(&mut app, "Cell Shield");

    app.update();

    assert_bound_count(app.world(), breaker, 1);
}

#[test]
fn active_cells_fire_stamps_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Damage",
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell Damage");

    app.update();

    assert_bound_count(app.world(), breaker, 1);
}

#[test]
fn two_active_cells_chips_both_stamp() {
    let mut app = test_app();

    let def_a = ChipDefinition::test_on(
        "Cell A",
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.1),
        })),
        5,
    );
    let def_b = ChipDefinition::test_on(
        "Cell B",
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.2),
        })),
        5,
    );
    insert_chip(&mut app, def_a);
    insert_chip(&mut app, def_b);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Cell A");
    select_chip(&mut app, "Cell B");

    app.update();

    assert_bound_count(app.world(), breaker, 2);
}
