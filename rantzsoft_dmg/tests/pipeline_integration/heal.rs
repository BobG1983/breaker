//! Group A/E — Heal-side pipeline behaviors 6 and 20.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    Dead, Dmgable, HealCap, HealDealt, Hp, Invulnerable, RantzDmgAppExt, RantzDmgPlugin,
};

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

// ── Behavior 6: heal path end-to-end — HealDealt raises current, clamped
//    by HealCap. ──

#[test]
fn heal_raises_current_below_starting_cap() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer:        None,
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        None,
            cap:           HealCap::Starting,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 8.0);
}

#[test]
fn heal_clamps_at_starting_when_cap_is_starting_even_if_max_is_higher() {
    // Edge case 6a.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer:        None,
            attributed_to: None,
            target:        victim,
            amount:        100.0,
            source:        None,
            cap:           HealCap::Starting,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 10.0);
}

#[test]
fn heal_clamps_at_max_when_cap_is_max() {
    // Edge case 6b.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer:        None,
            attributed_to: None,
            target:        victim,
            amount:        100.0,
            source:        None,
            cap:           HealCap::Max,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 20.0);
}

#[test]
fn heal_over_cap_short_circuits_when_current_already_at_max() {
    // Edge case 6c.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  20.0,
                starting: 10.0,
                max:      Some(20.0),
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Messages<HealDealt<TestT>>>()
        .write(HealDealt::<TestT> {
            healer:        None,
            attributed_to: None,
            target:        victim,
            amount:        1.0,
            source:        None,
            cap:           HealCap::Max,
            _marker:       PhantomData,
        });
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 20.0);
}

// ── Behavior 20: heal on Dead entity is a no-op; heal on Invulnerable
//    entity is a no-op. Edge 20a: a third unmarked victim DOES receive
//    the heal in the same tick — positive control proving apply_heal
//    actually ran. ──

#[test]
fn heal_on_dead_or_invulnerable_is_noop_and_live_victim_heals_same_tick() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let victim_a = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
            Dead,
        ))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
            Invulnerable,
        ))
        .id();
    let victim_c = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  3.0,
                starting: 10.0,
                max:      None,
            },
        ))
        .id();

    for target in [victim_a, victim_b, victim_c] {
        app.world_mut()
            .resource_mut::<Messages<HealDealt<TestT>>>()
            .write(HealDealt::<TestT> {
                healer: None,
                attributed_to: None,
                target,
                amount: 5.0,
                source: None,
                cap: HealCap::Starting,
                _marker: PhantomData,
            });
    }
    tick(&mut app);

    // Dead → no-op. Invulnerable → no-op. Unmarked → heal applies.
    // Having all three in the same tick proves apply_heal ran and the
    // skip on Dead/Invulnerable is a true filter (not a no-op pipeline).
    assert_f32_eq(app.world().get::<Hp>(victim_a).unwrap().current, 3.0);
    assert_f32_eq(app.world().get::<Hp>(victim_b).unwrap().current, 3.0);
    assert_f32_eq(app.world().get::<Hp>(victim_c).unwrap().current, 8.0);
}
