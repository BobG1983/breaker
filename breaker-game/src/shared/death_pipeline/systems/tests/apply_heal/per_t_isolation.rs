//! Behavior 31: Per-T queue isolation (`TestEntity` and `Salvo` side-by-side).

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::helpers::TestEntity;
use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    prelude::*,
    shared::death_pipeline::{
        heal_dealt::{HealCap, HealDealt},
        systems::apply_heal,
    },
};

/// Behavior 31: `HealDealt`<TestEntity> does not heal a Salvo entity.
/// Both monomorphizations are registered in the same app.
#[test]
fn heal_wrong_monomorphization_test_entity_message_does_not_heal_salvo() {
    // One-off harness — both monomorphizations registered.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<HealDealt<TestEntity>>();
    app.add_message::<HealDealt<Salvo>>();
    app.add_systems(FixedUpdate, apply_heal::<TestEntity>);
    app.add_systems(FixedUpdate, apply_heal::<Salvo>);

    // Spawn a Salvo-only entity (NO TestEntity marker on it).
    let salvo_entity = app
        .world_mut()
        .spawn((Salvo, Hp::new(10.0), KilledBy::default()))
        .id();
    // Pre-damage salvo to 5.0 so a potential heal would be visible.
    app.world_mut().get_mut::<Hp>(salvo_entity).unwrap().current = 5.0;

    // Write HealDealt<TestEntity> targeting the Salvo entity.
    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestEntity>>>()
        .write(HealDealt::<TestEntity> {
            healer:  None,
            target:  salvo_entity,
            amount:  5.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        });

    tick(&mut app);

    let hp = app.world().get::<Hp>(salvo_entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Salvo entity should not be healed by HealDealt<TestEntity>, got {}",
        hp.current
    );
}

/// Behavior 31 converse: `HealDealt`<Salvo> does not heal a `TestEntity`.
#[test]
fn heal_wrong_monomorphization_salvo_message_does_not_heal_test_entity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<HealDealt<TestEntity>>();
    app.add_message::<HealDealt<Salvo>>();
    app.add_systems(FixedUpdate, apply_heal::<TestEntity>);
    app.add_systems(FixedUpdate, apply_heal::<Salvo>);

    let test_entity = app
        .world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default()))
        .id();
    app.world_mut().get_mut::<Hp>(test_entity).unwrap().current = 5.0;

    app.world_mut()
        .resource_mut::<Messages<HealDealt<Salvo>>>()
        .write(HealDealt::<Salvo> {
            healer:  None,
            target:  test_entity,
            amount:  5.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        });

    tick(&mut app);

    let hp = app.world().get::<Hp>(test_entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "TestEntity should not be healed by HealDealt<Salvo>, got {}",
        hp.current
    );
}
