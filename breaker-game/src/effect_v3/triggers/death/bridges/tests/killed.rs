use bevy::prelude::*;

use super::helpers::{
    TestBoltDestroyedMessages, TestBreakerDestroyedMessages, TestCellDestroyedMessages,
    TestWallDestroyedMessages, bolt_death_test_app, breaker_death_test_app, cell_death_test_app,
    death_speed_tree, destroyed_bolt, destroyed_breaker, destroyed_cell, destroyed_wall,
    wall_death_test_app,
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

// -- Behavior 11 edge case: killer is None, Killed(Any) does NOT fire --

#[test]
fn killed_any_does_not_fire_when_killer_is_none_for_bolt_death() {
    let mut app = bolt_death_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();

    // This entity has Killed(Any) — should NOT fire because killer is None
    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestBoltDestroyedMessages(vec![destroyed_bolt(
        bolt_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "Killed(Any) should not fire when killer is None (environmental death)"
    );
}

// -- Behavior 12 edge case: Killed(Any) fires on killer for breaker death --

#[test]
fn killed_any_fires_on_killer_when_breaker_dies() {
    let mut app = breaker_death_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();

    // bolt_entity is the killer, and has Killed(Any) tree
    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Any),
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
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("killer bolt_entity should have EffectStack from Killed(Any)");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 13: Killed(Any) fires on killer when Cell dies --

#[test]
fn killed_any_fires_on_killer_when_cell_dies() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("bolt_entity (killer) should have EffectStack from Killed(Any)");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 13 edge case: Killed(Any) and Killed(Cell) both fire --

#[test]
fn killed_any_and_killed_specific_both_fire_when_cell_dies() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_any", Trigger::Killed(EntityKind::Any), 1.5),
            death_speed_tree("chip_cell", Trigger::Killed(EntityKind::Cell), 2.0),
        ]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("bolt_entity should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both Killed(Any) and Killed(Cell) should fire on the killer"
    );
}

// -- Behavior 14: Killed(Any) fires on killer when Wall dies --

#[test]
fn killed_any_fires_on_killer_when_wall_dies() {
    let mut app = wall_death_test_app();

    let wall_entity = app.world_mut().spawn_empty().id();

    let bolt_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestWallDestroyedMessages(vec![destroyed_wall(
        wall_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity)
        .expect("bolt_entity (killer) should have EffectStack from Killed(Any) on wall death");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 14 edge case: killer has no BoundEffects — no panic --

#[test]
fn killed_any_no_panic_when_killer_has_no_bound_effects() {
    let mut app = wall_death_test_app();

    let wall_entity = app.world_mut().spawn_empty().id();
    // killer has no BoundEffects
    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestWallDestroyedMessages(vec![destroyed_wall(
        wall_entity,
        Some(bolt_entity),
    )]));

    // Should not panic
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt_entity);
    assert!(
        stack.is_none(),
        "no BoundEffects should mean no EffectStack"
    );
}

// -- Behavior 15: Killed(Any) does NOT fire when killer is None --

#[test]
fn killed_any_does_not_fire_when_killer_is_none() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "Killed(Any) should not fire when killer is None"
    );
}

// -- Behavior 15 edge case: both Killed(Any) and DeathOccurred(Any) —
//    only DeathOccurred fires when killer is None --

#[test]
fn only_death_occurred_any_fires_when_killer_is_none() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_killed", Trigger::Killed(EntityKind::Any), 1.5),
            death_speed_tree("chip_death", Trigger::DeathOccurred(EntityKind::Any), 2.0),
        ]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack from DeathOccurred(Any)");
    assert_eq!(
        stack.len(),
        1,
        "Only DeathOccurred(Any) should fire; Killed(Any) should not (killer is None)"
    );
}

// -- Behavior 17: Killed(Any) and Killed(specific) both fire --

#[test]
fn killed_any_and_killed_specific_both_fire_for_cell_death() {
    let mut app = cell_death_test_app();

    let cell_entity = app.world_mut().spawn_empty().id();

    let killer_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_cell", Trigger::Killed(EntityKind::Cell), 1.5),
            death_speed_tree("chip_any", Trigger::Killed(EntityKind::Any), 2.0),
        ]))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(killer_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(killer_entity)
        .expect("killer_entity should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both Killed(Cell) and Killed(Any) should fire on the killer"
    );
}
