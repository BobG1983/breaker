//! Group B — Killing-blow and kill-attribution end-to-end, behaviors 7–10.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageDealt, Destroyed, Dmgable, Hp, RantzDmgAppExt, RantzDmgPlugin};
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

// ── Behavior 7: first-kill-wins across same-tick multi-hit that crosses
//    zero. ──

#[test]
fn first_kill_wins_when_later_hit_crosses_zero() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(5.0, 5.0))))
        .id();
    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    // Non-lethal first, lethal second — dealer_b is credited.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:  Some(dealer_a),
            target:  victim,
            amount:  3.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:  Some(dealer_b),
            target:  victim,
            amount:  10.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].killer, Some(dealer_b));
    assert!(app.world().get_entity(victim).is_err());
}

#[test]
fn first_kill_wins_with_three_hits_third_crosses_zero() {
    // Edge case 7a.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(5.0, 5.0))))
        .id();
    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();
    let dealer_c = app.world_mut().spawn_empty().id();

    for dealer in [dealer_a, dealer_b, dealer_c] {
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  Some(dealer),
                target:  victim,
                amount:  4.0,
                source:  None,
                _marker: PhantomData,
            });
    }
    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].killer, Some(dealer_c));
    assert!(app.world().get_entity(victim).is_err());
}

// ── Behavior 8: no Destroyed emission on idle tick. ──

#[test]
fn idle_tick_emits_no_destroyed_and_leaves_hp_unchanged() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

    tick(&mut app);

    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
    assert!(app.world().get_entity(victim).is_ok());
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 10.0);
}

#[test]
fn three_idle_ticks_all_yield_empty_destroyed_and_keep_hp() {
    // Edge case 8a.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

    for _ in 0..3 {
        tick(&mut app);
        let count = app
            .world_mut()
            .resource_mut::<Messages<Destroyed<TestT>>>()
            .drain()
            .count();
        assert_eq!(count, 0);
    }
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 10.0);
}

// ── Behavior 9: entity reaches HP 0 via detect_deaths without going
//    through apply_damage. ──

#[test]
fn zero_hp_entity_is_destroyed_with_killer_none() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(0.0), Position2D(Vec2::new(7.0, 8.0))))
        .id();

    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, victim);
    assert!(destroyed[0].killer.is_none());
    assert_f32_eq(destroyed[0].victim_pos.x, 7.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 8.0);
    assert!(destroyed[0].killer_pos.is_none());
    assert!(app.world().get_entity(victim).is_err());
}

#[test]
fn negative_hp_entity_is_destroyed_with_killer_none() {
    // Edge case 9a.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  -5.0,
                starting: 10.0,
                max:      None,
            },
            Position2D(Vec2::new(0.0, 0.0)),
        ))
        .id();

    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert!(destroyed[0].killer.is_none());
    assert!(app.world().get_entity(victim).is_err());
}

// ── Behavior 10: same-tick duplicate killing-blow dealers result in
//    exactly one Destroyed, killer is the first dealer. ──

#[test]
fn many_hit_victim_dies_exactly_once_crediting_first_lethal_dealer() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Position2D(Vec2::new(0.0, 0.0))))
        .id();
    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    // Two identical lethal amounts from distinct dealers in the same tick.
    for dealer in [dealer_a, dealer_b] {
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  Some(dealer),
                target:  victim,
                amount:  10.0,
                source:  None,
                _marker: PhantomData,
            });
    }
    tick(&mut app);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].killer, Some(dealer_a));
    assert!(app.world().get_entity(victim).is_err());
}
