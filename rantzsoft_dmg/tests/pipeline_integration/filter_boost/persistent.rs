//! Persistent-lane source-filter coverage: a dealer with a populated
//! filtered persistent entry emits `DamageDealt<TestT>` messages with various
//! `source` values.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageBoostStack, DamageDealt, Hp, RantzDmgAppExt, SourceId};

use super::helpers::{TestT, app_with_plugin, assert_f32_eq, tick};

// ── Behavior 27: persistent filtered boost amplifies a matching DamageDealt<T> ──

#[test]
fn persistent_filtered_boost_amplifies_matching_damage_dealt() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_filtered(
            SourceId::from("src:burnout"),
            2.0,
            SourceId::from("protocol:burnout"),
        );
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
            source:        Some(SourceId::from("protocol:burnout")),
            _marker:       PhantomData,
        });
    tick(&mut app);

    // 5.0 base * 2.0 boost (matches filter) = 10.0 damage; 100 - 10 = 90.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 90.0);
}

#[test]
fn persistent_filtered_boost_survives_to_amplify_a_second_matching_emission() {
    // Edge case for Behavior 27: persistent filtered entries are NOT
    // one-shots — they survive across ticks.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_filtered(
            SourceId::from("src:burnout"),
            2.0,
            SourceId::from("protocol:burnout"),
        );
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
            source:        Some(SourceId::from("protocol:burnout")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 90.0);

    // Second identical message — persistent entry still amplifies.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("protocol:burnout")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 80.0);
}

// ── Behavior 28: persistent filtered boost does NOT amplify a non-matching DamageDealt<T> ──

#[test]
fn persistent_filtered_boost_does_not_amplify_non_matching_damage_dealt() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_filtered(
            SourceId::from("src:burnout"),
            2.0,
            SourceId::from("protocol:burnout"),
        );
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
            source:        Some(SourceId::from("protocol:debt_collector")),
            _marker:       PhantomData,
        });
    tick(&mut app);

    // 5.0 base; no boost (filter does not match emission); 100 - 5 = 95.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 95.0);
}

#[test]
fn persistent_filtered_boost_survives_non_matching_emission_then_amplifies_match() {
    // Edge case for Behavior 28: pins (a) non-matching emission is
    // unaffected, AND (b) non-matching emission did not drop the
    // persistent entry.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_filtered(
            SourceId::from("src:burnout"),
            2.0,
            SourceId::from("protocol:burnout"),
        );
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    // Tick 1 — non-matching emission, no boost applied.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("protocol:debt_collector")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 95.0);

    // Tick 2 — matching emission, persistent entry must still be present.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("protocol:burnout")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 85.0);
}
