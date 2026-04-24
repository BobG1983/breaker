//! Group A — Full-pipeline end-to-end, damage-side behaviors 1–5.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageBoostStack, DamageDealt, Destroyed, Dmgable, Hp, Invulnerable, RantzDmgAppExt,
    RantzDmgPlugin, SourceId, VulnerableStack,
};
use rantzsoft_spatial2d::components::Position2D;

#[derive(Component)]
struct TestT;
impl Dmgable for TestT {}

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

fn app_with_plugin() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

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

// ── Behavior 2: DamageBoostStack persistent + VulnerableStack persistent
//    multiply damage across the pipeline. ──

#[test]
fn persistent_damage_boost_and_vulnerable_multiply_end_to_end() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:boost"), 2.0);
        app.world_mut().spawn(s).id()
    };
    let victim = {
        let mut v = VulnerableStack::default();
        v.add(SourceId::from("src:vuln"), 2.5);
        app.world_mut()
            .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(0.0, 0.0)), v))
            .id()
    };

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        1.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    // 1.0 * 2.0 (boost) * 2.5 (vulnerable) = 5.0 damage; HP 10 - 5 = 5.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 5.0);
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
    assert!(app.world().get_entity(victim).is_ok());
}

#[test]
fn persistent_boost_and_vulnerable_kill_when_amount_pushes_to_lethal() {
    // Edge case 2a: 2.0 base * 2.0 * 2.5 = 10.0 lethal damage.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:boost"), 2.0);
        app.world_mut().spawn(s).id()
    };
    let victim = {
        let mut v = VulnerableStack::default();
        v.add(SourceId::from("src:vuln"), 2.5);
        app.world_mut()
            .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(0.0, 0.0)), v))
            .id()
    };

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        2.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 1);
    assert!(app.world().get_entity(victim).is_err());
    // NOTE: do not read Hp after despawn — the entity no longer exists.
}

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

// ── Behavior 5: Invulnerable target short-circuits damage end-to-end. ──

#[test]
fn invulnerable_target_short_circuits_damage_end_to_end() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot(100.0);
        app.world_mut().spawn(s).id()
    };
    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp::new(10.0),
            Invulnerable,
            Position2D(Vec2::new(0.0, 0.0)),
        ))
        .id();

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

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 10.0);
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
    assert!(app.world().get_entity(victim).is_ok());
}
