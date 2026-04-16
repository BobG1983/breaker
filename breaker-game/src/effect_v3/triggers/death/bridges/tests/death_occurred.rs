use bevy::prelude::*;

use super::helpers::{
    TestBoltDestroyedMessages, TestBreakerDestroyedMessages, TestCellDestroyedMessages,
    bolt_death_test_app, breaker_death_test_app, cell_death_test_app, death_speed_tree,
    destroyed_bolt, destroyed_breaker, destroyed_cell,
};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EntityKind, Trigger},
    },
    prelude::*,
};

// -- Behavior 10: DeathOccurred(Any) fires on all entities when Cell dies --

#[test]
fn death_occurred_any_fires_on_all_entities_when_cell_dies() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            1.5,
        )]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_b",
            Trigger::DeathOccurred(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack after DeathOccurred(Any)");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("entity_b should have EffectStack after DeathOccurred(Any)");
    assert_eq!(stack_b.len(), 1);
}

// -- Behavior 10 edge case: specific kind still works alongside Any --

#[test]
fn death_occurred_specific_fires_alongside_any_when_cell_dies() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_any", Trigger::DeathOccurred(EntityKind::Any), 1.5),
            death_speed_tree("chip_cell", Trigger::DeathOccurred(EntityKind::Cell), 2.0),
        ]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both Any and Cell DeathOccurred gates should fire"
    );
}

// -- Behavior 11: DeathOccurred(Any) fires when Bolt dies --

#[test]
fn death_occurred_any_fires_when_bolt_dies() {
    let mut app = bolt_death_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            2.0,
        )]))
        .id();

    app.insert_resource(TestBoltDestroyedMessages(vec![destroyed_bolt(
        bolt_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after bolt DeathOccurred(Any)");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 12: DeathOccurred(Any) fires when Breaker dies --

#[test]
fn death_occurred_any_fires_when_breaker_dies() {
    let mut app = breaker_death_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestBreakerDestroyedMessages(vec![destroyed_breaker(
        breaker_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after breaker DeathOccurred(Any)");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 16: DeathOccurred(Any) and DeathOccurred(specific) both fire --

#[test]
fn death_occurred_any_and_specific_both_fire_for_cell_death() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_cell", Trigger::DeathOccurred(EntityKind::Cell), 1.5),
            death_speed_tree("chip_any", Trigger::DeathOccurred(EntityKind::Any), 2.0),
        ]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both DeathOccurred(Cell) and DeathOccurred(Any) should fire"
    );
}

// -- Behavior 18: DeathOccurred(Any) no-op when no Destroyed messages --

#[test]
fn death_occurred_any_no_op_without_destroyed_messages() {
    let mut app = cell_death_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            1.5,
        )]))
        .id();

    // No messages injected
    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "no EffectStack should exist when no Destroyed messages are sent"
    );
}
