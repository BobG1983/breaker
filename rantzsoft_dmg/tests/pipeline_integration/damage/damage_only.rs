//! Behavior 1 — damage-only pipeline lands HP decrement and emits
//! `Destroyed` for the killing blow.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageDealt, Destroyed, Hp, RantzDmgAppExt};
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{TestT, app_with_plugin, assert_f32_eq, tick};

// ── Behavior 1: damage-only path lands final HP decrement and emits
//    `Destroyed` for the killing blow. ──

#[test]
fn damage_only_pipeline_emits_destroyed_for_killing_blow() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        10.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, victim);
    assert_eq!(destroyed[0].killer, Some(dealer));
    assert_f32_eq(destroyed[0].victim_pos.x, 1.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 2.0);
    assert!(destroyed[0].killer_pos.is_none());
    assert!(app.world().get_entity(victim).is_err());
}

#[test]
fn damage_only_pipeline_overkill_still_emits_single_destroyed() {
    // Edge case 1a: overkill amount.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        15.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, victim);
    assert_eq!(destroyed[0].killer, Some(dealer));
    assert!(app.world().get_entity(victim).is_err());
}
