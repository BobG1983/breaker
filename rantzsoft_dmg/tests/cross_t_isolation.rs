//! P7 crate integration tests — cross-`T` isolation (behaviors 11–15).
//!
//! Each test registers two distinct `Dmgable` marker types (`T1`, `T2`)
//! and pins that per-`T` pipelines do not bleed into each other. This
//! file is its own integration-test binary; Cargo compiles it separately
//! from `pipeline_integration.rs`.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "integration tests use unwrap/expect/panic for assertion clarity"
    )
)]

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageBoostStack, DamageDealt, Destroyed, Dmgable, HealCap, HealDealt, Hp, Invulnerable,
    RantzDmgAppExt, RantzDmgPlugin, SourceId,
};
use rantzsoft_spatial2d::components::Position2D;

#[derive(Component)]
struct T1;
impl Dmgable for T1 {}

#[derive(Component)]
struct T2;
impl Dmgable for T2 {}

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

fn app_with_both_types() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<T1>().register_dmgable::<T2>();
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// ── Behavior 11: DamageDealt<T1> does not affect a T2 target. ──

#[test]
fn damage_dealt_t1_does_not_affect_t2_target() {
    let mut app = app_with_both_types();

    let victim_a = app.world_mut().spawn((T1, Hp::new(10.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(10.0))).id();

    // T1 message targeting a T2 entity — apply_damage::<T1> query filters
    // With<T1>, so victim_b is skipped.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  None,
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 10.0);
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 10.0);

    let t1_destroyed = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T1>>>()
        .drain()
        .count();
    let t2_destroyed = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T2>>>()
        .drain()
        .count();
    assert_eq!(t1_destroyed, 0);
    assert_eq!(t2_destroyed, 0);
}

#[test]
fn damage_dealt_t2_affects_t2_target_and_leaves_t1_alone() {
    // Edge case 11a.
    let mut app = app_with_both_types();

    let victim_a = app.world_mut().spawn((T1, Hp::new(10.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(10.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  None,
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 5.0);
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 10.0);
}

// ── Behavior 12: HealDealt<T1> does not affect a T2 target. ──

#[test]
fn heal_dealt_t1_does_not_affect_t2_target() {
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((
            T1,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((
            T2,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<T1>>>()
        .write(HealDealt::<T1> {
            healer:  None,
            target:  victim_b,
            amount:  5.0,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 3.0);
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 3.0);
}

#[test]
fn heal_dealt_t2_affects_t2_target() {
    // Edge case 12a.
    let mut app = app_with_both_types();

    let victim_b = app
        .world_mut()
        .spawn((
            T2,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<T2>>>()
        .write(HealDealt::<T2> {
            healer:  None,
            target:  victim_b,
            amount:  5.0,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 8.0);
}

// ── Behavior 13: dual-T kill in one tick — each victim dies in its own
//    Destroyed<T> queue, killers attributed independently. ──

#[test]
fn dual_t_kill_emits_one_destroyed_per_type_with_correct_killer() {
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((T1, Hp::new(5.0), Position2D(Vec2::new(1.0, 1.0))))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((T2, Hp::new(5.0), Position2D(Vec2::new(9.0, 9.0))))
        .id();
    let dealer_1 = app.world_mut().spawn_empty().id();
    let dealer_2 = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer_1),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  Some(dealer_2),
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let t1_destroyed: Vec<Destroyed<T1>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T1>>>()
        .drain()
        .collect();
    let t2_destroyed: Vec<Destroyed<T2>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T2>>>()
        .drain()
        .collect();
    assert_eq!(t1_destroyed.len(), 1);
    assert_eq!(t1_destroyed[0].victim, victim_a);
    assert_f32_eq(t1_destroyed[0].victim_pos.x, 1.0);
    assert_f32_eq(t1_destroyed[0].victim_pos.y, 1.0);
    assert_eq!(t1_destroyed[0].killer, Some(dealer_1));

    assert_eq!(t2_destroyed.len(), 1);
    assert_eq!(t2_destroyed[0].victim, victim_b);
    assert_f32_eq(t2_destroyed[0].victim_pos.x, 9.0);
    assert_f32_eq(t2_destroyed[0].victim_pos.y, 9.0);
    assert_eq!(t2_destroyed[0].killer, Some(dealer_2));

    assert!(app.world().get_entity(victim_a).is_err());
    assert!(app.world().get_entity(victim_b).is_err());
}

#[test]
fn single_t1_kill_leaves_t2_victim_alive_and_t2_destroyed_queue_empty() {
    // Edge case 13a.
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((T1, Hp::new(5.0), Position2D(Vec2::new(1.0, 1.0))))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((T2, Hp::new(5.0), Position2D(Vec2::new(9.0, 9.0))))
        .id();
    let dealer_1 = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer_1),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let t1_destroyed: Vec<Destroyed<T1>> = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T1>>>()
        .drain()
        .collect();
    let t2_destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<T2>>>()
        .drain()
        .count();
    assert_eq!(t1_destroyed.len(), 1);
    assert_eq!(t1_destroyed[0].killer, Some(dealer_1));
    assert_eq!(t2_destroyed_count, 0);
    assert!(app.world().get_entity(victim_b).is_ok());
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 5.0);
}

// ── Behavior 14: DamageBoostStack on a dealer is indiscriminate of
//    pipeline T. ──

#[test]
fn persistent_boost_on_single_dealer_multiplies_both_t_queues() {
    let mut app = app_with_both_types();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:boost"), 2.0);
        app.world_mut().spawn(s).id()
    };
    let victim_a = app.world_mut().spawn((T1, Hp::new(100.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(100.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  Some(dealer),
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    // 100 - 5 * 2 = 90, for both victims independently.
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 90.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 90.0);
}

#[test]
fn one_shot_on_single_dealer_drained_by_whichever_t_system_runs_first() {
    // Edge case 14a: DOCUMENTED IRREGULARITY — a single one-shot is
    // consumed by whichever of `apply_damage_boosts::<T1>` or
    // `apply_damage_boosts::<T2>` runs first this tick. Rather than assume
    // the order, assert the order-independent aggregate HP drop across
    // both victims. One victim takes 5 base, the other takes 5 * 2 = 10.
    let mut app = app_with_both_types();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot(2.0);
        app.world_mut().spawn(s).id()
    };
    let victim_a = app.world_mut().spawn((T1, Hp::new(100.0))).id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(100.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  Some(dealer),
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  Some(dealer),
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    let hp_a = app.world().get::<Hp>(victim_a).unwrap().current;
    let hp_b = app.world().get::<Hp>(victim_b).unwrap().current;
    // Total remaining HP is 200 - (5 + 10) = 185, regardless of order.
    assert_f32_eq(hp_a + hp_b, 185.0);
}

// ── Behavior 15: Invulnerable on a T1 victim does NOT shield a T2 victim. ──

#[test]
fn invulnerable_on_t1_does_not_shield_t2_victim() {
    let mut app = app_with_both_types();

    let victim_a = app
        .world_mut()
        .spawn((T1, Hp::new(10.0), Invulnerable))
        .id();
    let victim_b = app.world_mut().spawn((T2, Hp::new(10.0))).id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  None,
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  None,
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 10.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 5.0);
}

#[test]
fn invulnerable_on_t2_does_not_shield_t1_victim() {
    // Edge case 15a: swap the invulnerable marker.
    let mut app = app_with_both_types();

    let victim_a = app.world_mut().spawn((T1, Hp::new(10.0))).id();
    let victim_b = app
        .world_mut()
        .spawn((T2, Hp::new(10.0), Invulnerable))
        .id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T1>>>()
        .write(DamageDealt::<T1> {
            dealer:  None,
            target:  victim_a,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<T2>>>()
        .write(DamageDealt::<T2> {
            dealer:  None,
            target:  victim_b,
            amount:  5.0,
            source:  None,
            _marker: PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 5.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 10.0);
}
