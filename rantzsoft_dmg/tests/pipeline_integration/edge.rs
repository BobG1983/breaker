//! Group E — Numeric / multi-entity edge behaviors 21–22.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageBoostStack, DamageDealt, Destroyed, Dmgable, HealCap, HealDealt, Hp, Invulnerable,
    RantzDmgAppExt, RantzDmgPlugin,
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

/// Spawn the five-victim batch used by B22 and 22a: 3 damage targets,
/// 2 heal targets. Returns `[victim_1..victim_5]`.
fn spawn_five_victims(app: &mut App) -> [Entity; 5] {
    let victim_1 = app.world_mut().spawn((TestT, Hp::new(100.0))).id();
    let victim_2 = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  2.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();
    let victim_3 = app
        .world_mut()
        .spawn((TestT, Hp::new(5.0), Position2D(Vec2::new(3.0, 4.0))))
        .id();
    let victim_4 = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  1.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();
    let victim_5 = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  9.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();
    [victim_1, victim_2, victim_3, victim_4, victim_5]
}

/// Write the five batch messages (3 damage, 2 heal) for the B22 scenario.
fn send_batch_messages(app: &mut App, victims: [Entity; 5]) {
    let [v1, v2, v3, v4, v5] = victims;
    for (target, amount) in [(v1, 10.0), (v2, 1.0), (v3, 10.0)] {
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer: None,
                target,
                amount,
                source: None,
                _marker: PhantomData,
            });
    }
    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer:  None,
            target:  v4,
            amount:  3.0,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer:  None,
            target:  v5,
            amount:  100.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        });
}

// ── Behavior 21: Invulnerable + damage in the same tick — boost one-shots
//    STILL drain before invulnerable_filter zeros the amount. ──

#[test]
fn invulnerable_victim_drains_boost_one_shot_in_same_tick() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot(5.0);
        app.world_mut().spawn(s).id()
    };
    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Invulnerable))
        .id();

    // Tick 1: Invulnerable zeros the applied damage, but the boost
    // one-shot is already consumed by apply_damage_boosts.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:  Some(dealer),
            target:  victim,
            amount:  2.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 10.0);

    // Remove invulnerable, send another identical message. If the
    // one-shot correctly drained, tick 2 applies 2.0 * 1.0 = 2.0 damage,
    // NOT 2.0 * 5.0 = 10.0.
    app.world_mut().entity_mut(victim).remove::<Invulnerable>();
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:  Some(dealer),
            target:  victim,
            amount:  2.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 8.0);
}

#[test]
fn nan_one_shot_produces_nan_hp_garbage_in_garbage_out() {
    // Edge case 21a: documented "garbage in, garbage out" contract.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot(f32::NAN);
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:  Some(dealer),
            target:  victim,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert!(app.world().get::<Hp>(victim).unwrap().current.is_nan());
}

// ── Behavior 22: multi-entity batch — three damage, two heal, one kill. ──

#[test]
fn multi_entity_batch_resolves_each_victim_independently_in_one_tick() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let [v1, v2, v3, v4, v5] = spawn_five_victims(&mut app);
    send_batch_messages(&mut app, [v1, v2, v3, v4, v5]);
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(v1).unwrap().current, 90.0);
    assert_f32_eq(app.world().get::<Hp>(v2).unwrap().current, 1.0);
    assert!(app.world().get_entity(v3).is_err());
    assert_f32_eq(app.world().get::<Hp>(v4).unwrap().current, 4.0);
    assert_f32_eq(app.world().get::<Hp>(v5).unwrap().current, 20.0);

    let destroyed: Vec<Destroyed<TestT>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect();
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, v3);
    assert_f32_eq(destroyed[0].victim_pos.x, 3.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 4.0);
    assert!(destroyed[0].killer.is_none());
    assert!(destroyed[0].killer_pos.is_none());
}

#[test]
fn multi_entity_batch_idle_second_tick_has_no_residual_processing() {
    // Edge case 22a: after the batch resolves in tick 1, tick 2 with no
    // new messages leaves all live victims unchanged and emits no further
    // Destroyed messages.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let [v1, v2, v3, v4, v5] = spawn_five_victims(&mut app);
    send_batch_messages(&mut app, [v1, v2, v3, v4, v5]);
    tick(&mut app);

    // Drain the tick-1 Destroyed message so the residual check is clean.
    let _ = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();

    // Tick 2 with no new messages.
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(v1).unwrap().current, 90.0);
    assert_f32_eq(app.world().get::<Hp>(v2).unwrap().current, 1.0);
    assert!(app.world().get_entity(v3).is_err());
    assert_f32_eq(app.world().get::<Hp>(v4).unwrap().current, 4.0);
    assert_f32_eq(app.world().get::<Hp>(v5).unwrap().current, 20.0);

    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
}
