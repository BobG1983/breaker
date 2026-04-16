use std::marker::PhantomData;

use bevy::prelude::*;

use super::helpers::death_speed_tree;
use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        types::{EntityKind, Trigger},
    },
    prelude::*,
};

// ==========================================================================
// Section 4: Death Bridge — on_salvo_destroyed
// ==========================================================================

#[derive(Resource, Default)]
struct TestSalvoDestroyedMessages(Vec<Destroyed<Salvo>>);

fn inject_salvo_destroyed(
    messages: Res<TestSalvoDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Salvo>>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

fn salvo_death_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<Destroyed<Salvo>>()
        .with_resource::<TestSalvoDestroyedMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_salvo_destroyed.before(super::super::system::on_salvo_destroyed),
                super::super::system::on_salvo_destroyed,
            ),
        )
        .build()
}

fn destroyed_salvo(victim: Entity, killer: Option<Entity>) -> Destroyed<Salvo> {
    Destroyed {
        victim,
        killer,
        victim_pos: Vec2::ZERO,
        killer_pos: killer.map(|_| Vec2::ZERO),
        _marker: PhantomData,
    }
}

// -- Behavior 11: Died trigger fires on Salvo victim entity --

#[test]
fn died_trigger_fires_on_salvo_victim() {
    let mut app = salvo_death_test_app();

    let killer_entity = app.world_mut().spawn_empty().id();

    let salvo_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Died,
            1.5,
        )]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(killer_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity)
        .expect("salvo victim should have EffectStack from Died trigger");
    assert_eq!(stack.len(), 1);
}

#[test]
fn died_trigger_no_panic_when_salvo_victim_has_no_bound_effects() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let killer_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(killer_entity),
    )]));

    // Should not panic
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity);
    assert!(stack.is_none(), "no BoundEffects means no EffectStack");
}

// -- Behavior 12: Killed(Salvo) fires on killer entity --

#[test]
fn killed_salvo_fires_on_killer_entity() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Salvo),
            2.0,
        )]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(breaker_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("killer should have EffectStack from Killed(Salvo)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn killed_salvo_does_not_fire_when_killer_is_none() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Salvo),
            1.5,
        )]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "Killed(Salvo) should not fire when killer is None"
    );
}

// -- Behavior 13: Killed(Any) fires on killer entity when Salvo dies --

#[test]
fn killed_any_fires_on_killer_when_salvo_dies() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Killed(EntityKind::Any),
            1.5,
        )]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(breaker_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("killer should have EffectStack from Killed(Any)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn killed_salvo_and_killed_any_both_fire_on_killer() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_salvo", Trigger::Killed(EntityKind::Salvo), 1.5),
            death_speed_tree("chip_any", Trigger::Killed(EntityKind::Any), 2.0),
        ]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(breaker_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("killer should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both Killed(Salvo) and Killed(Any) should fire on the killer"
    );
}

// -- Behavior 14: DeathOccurred(Salvo) fires globally --

#[test]
fn death_occurred_salvo_fires_globally_on_all_entities() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Salvo),
            1.5,
        )]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_b",
            Trigger::DeathOccurred(EntityKind::Salvo),
            1.5,
        )]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack after DeathOccurred(Salvo)");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("entity_b should have EffectStack after DeathOccurred(Salvo)");
    assert_eq!(stack_b.len(), 1);
}

#[test]
fn death_occurred_salvo_no_panic_on_entity_without_bound_effects() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let _naked_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        None,
    )]));

    // Should not panic
    tick(&mut app);
}

// -- Behavior 15: DeathOccurred(Any) fires globally when Salvo dies --

#[test]
fn death_occurred_any_fires_when_salvo_dies() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Any),
            2.0,
        )]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after DeathOccurred(Any) from salvo death");
    assert_eq!(stack.len(), 1);
}

#[test]
fn death_occurred_salvo_and_any_both_fire_on_same_entity() {
    let mut app = salvo_death_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            death_speed_tree("chip_salvo", Trigger::DeathOccurred(EntityKind::Salvo), 1.5),
            death_speed_tree("chip_any", Trigger::DeathOccurred(EntityKind::Any), 2.0),
        ]))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        None,
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both DeathOccurred(Salvo) and DeathOccurred(Any) should fire"
    );
}

// -- Behavior 16: StagedEffects path for salvo death --

#[test]
fn salvo_death_fires_staged_entry_and_consumes_it() {
    let mut app = salvo_death_test_app();
    let killer_entity = app.world_mut().spawn_empty().id();
    let staged_fire_tree = death_speed_tree("chip_a", Trigger::Died, 1.5).1;

    let salvo_entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![]),
            StagedEffects(vec![("chip_a".to_string(), staged_fire_tree)]),
        ))
        .id();

    app.insert_resource(TestSalvoDestroyedMessages(vec![destroyed_salvo(
        salvo_entity,
        Some(killer_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity)
        .expect("staged When(Died, Fire) must have fired on the salvo victim");
    assert_eq!(stack.len(), 1);

    let staged = app.world().get::<StagedEffects>(salvo_entity).unwrap();
    assert!(staged.0.is_empty(), "staged entry should be consumed");
    let bound = app.world().get::<BoundEffects>(salvo_entity).unwrap();
    assert!(bound.0.is_empty(), "BoundEffects must not be touched");
}

// -- Behavior 17: No Destroyed<Salvo> — on_salvo_destroyed is a no-op --

#[test]
fn on_salvo_destroyed_is_noop_without_messages() {
    let mut app = salvo_death_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::DeathOccurred(EntityKind::Salvo),
            1.5,
        )]))
        .id();

    // No messages injected
    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "no EffectStack should exist when no Destroyed<Salvo> messages are sent"
    );
}
