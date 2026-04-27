use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::*,
    helpers::{OtherT, TestT, app_with_plugin, tick},
};
use crate::messages::{DamageDealt, DespawnEntity, Destroyed, HealDealt, KillYourself};

// ── Behavior 139: register_dmgable::<TestT> adds Messages<DamageDealt<TestT>> ──

#[test]
fn register_dmgable_adds_damage_dealt_resource() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
}

// ── Behavior 140: register_dmgable adds HealDealt, KillYourself, Destroyed ──

#[test]
fn register_dmgable_adds_all_four_per_t_resources() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<HealDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<KillYourself<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<Destroyed<TestT>>>()
    );
}

#[test]
fn without_register_dmgable_none_of_the_four_resources_exist() {
    // Edge case 140a.
    let app = app_with_plugin();
    assert!(
        !app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<HealDealt<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<KillYourself<TestT>>>()
    );
    assert!(
        !app.world()
            .contains_resource::<Messages<Destroyed<TestT>>>()
    );
}

// ── Behavior 141: returns &mut Self for chaining ──

#[test]
fn returns_mut_self_for_chaining() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>().register_dmgable::<OtherT>();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<OtherT>>>()
    );
}

// ── Behavior 142: per-T queues are isolated ──

#[test]
fn per_t_queues_are_isolated() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>().register_dmgable::<OtherT>();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        None,
            attributed_to: None,
            target:        Entity::PLACEHOLDER,
            amount:        1.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let drained_t = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .drain()
        .count();
    let drained_u = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<OtherT>>>()
        .drain()
        .count();
    assert_eq!(drained_t, 1);
    assert_eq!(drained_u, 0);
}

// ── Behavior 143: single call registers all four resources ──

#[test]
fn single_call_registers_all_four_resources() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<HealDealt<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<KillYourself<TestT>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<Destroyed<TestT>>>()
    );
}

#[test]
fn single_call_chains_with_unrelated_add_message() {
    // Edge case 143a.
    #[derive(Message)]
    struct Unrelated;

    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>().add_message::<Unrelated>();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<TestT>>>()
    );
    assert!(app.world().contains_resource::<Messages<Unrelated>>());
}

// Keep DespawnEntity import exercised — the end-to-end tests verify
// despawn behavior indirectly; this assertion keeps the type visible.
#[test]
fn despawn_entity_message_exists_after_plugin_add() {
    let app = app_with_plugin();
    assert!(app.world().contains_resource::<Messages<DespawnEntity>>());
}
