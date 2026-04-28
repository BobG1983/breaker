//! Behaviors 3 & 4 — one-shots on `DamageBoostStack` and `VulnerableStack`
//! drain after one tick; persistent entries survive across ticks.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageBoostStack, DamageDealt, Destroyed, Hp, RantzDmgAppExt, SourceId, VulnerableStack,
};

use super::helpers::{TestT, app_with_plugin, assert_f32_eq, tick};

// ── Behavior 3: DamageBoostStack one-shots drain after one tick,
//    persistent survives. ──

#[test]
fn damage_boost_one_shot_drains_persistent_survives_across_ticks() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:persist"), 2.0);
        s.add_one_shot(3.0);
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    // Tick 1: amount 5.0 * 2.0 (persistent) * 3.0 (one-shot) = 30.0.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 70.0);

    // Tick 2: identical message; one-shot drained, so 5.0 * 2.0 = 10.0.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 60.0);

    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
}

#[test]
fn damage_boost_persistent_is_not_drained_across_idle_tick() {
    // Edge case 3a: tick with no new damage does not alter HP; persistent
    // multiplier still in place.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:persist"), 2.0);
        s.add_one_shot(3.0);
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 70.0);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 60.0);

    // Third tick: no new message. HP unchanged.
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 60.0);
}

// ── Behavior 4: VulnerableStack one-shots drain after one tick,
//    persistent survives. ──

#[test]
fn vulnerable_one_shot_drains_persistent_survives_across_ticks() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = app.world_mut().spawn_empty().id();
    let victim = {
        let mut v = VulnerableStack::default();
        v.add(SourceId::from("src:vp"), 2.0);
        v.add_one_shot(3.0);
        app.world_mut().spawn((TestT, Hp::new(100.0), v)).id()
    };

    // Tick 1: 5.0 * 2.0 * 3.0 = 30.0 damage → 70 HP.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 70.0);

    // Tick 2: one-shot drained, 5.0 * 2.0 = 10.0 → 60 HP.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 60.0);
}

#[test]
fn vulnerable_persistent_is_not_drained_across_idle_tick() {
    // Edge case 4a: third idle tick leaves HP unchanged.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = app.world_mut().spawn_empty().id();
    let victim = {
        let mut v = VulnerableStack::default();
        v.add(SourceId::from("src:vp"), 2.0);
        v.add_one_shot(3.0);
        app.world_mut().spawn((TestT, Hp::new(100.0), v)).id()
    };

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 60.0);

    // Idle tick.
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 60.0);
}
