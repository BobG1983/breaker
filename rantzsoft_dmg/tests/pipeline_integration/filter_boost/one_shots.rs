//! One-shot-lane source-filter coverage and combined filterless+filtered
//! persistent-lane scenarios.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{DamageBoostStack, DamageDealt, Hp, RantzDmgAppExt, SourceId};

use super::helpers::{TestT, app_with_plugin, assert_f32_eq, tick};

// ── Behavior 29: filtered one-shot is NOT consumed by non-matching, then IS consumed and amplifies a matching one ──

#[test]
fn filtered_one_shot_preserved_on_miss_then_drained_on_match() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot_filtered(3.0, SourceId::from("protocol:burnout"));
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    // Tick A — non-matching emission.
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

    // Tick B — matching emission; one-shot survived A and is consumed in B.
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
    // 95 - (5.0 * 3.0 = 15.0) = 80.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 80.0);
}

#[test]
fn filtered_one_shot_third_matching_emission_no_longer_amplifies() {
    // Edge case for Behavior 29: full lifecycle — preserved-on-miss,
    // drained-on-match, gone-on-next-match.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot_filtered(3.0, SourceId::from("protocol:burnout"));
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    // Tick A — non-matching, preserved.
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

    // Tick B — matching, drained.
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

    // Tick C — matching again, but one-shot is gone — no boost applied.
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
    // 80 - 5.0 = 75.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 75.0);
}

// ── Behavior 30: filtered one-shot with `source: None` emission is preserved ──

#[test]
fn filtered_one_shot_none_emission_does_not_amplify() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot_filtered(3.0, SourceId::from("protocol:burnout"));
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

    // 5.0 base; no boost (filtered one-shot does not match None emission);
    // 100 - 5 = 95.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 95.0);
}

#[test]
fn filtered_one_shot_preserved_after_none_emission_then_amplifies_match() {
    // Edge case for Behavior 30: pins that the None-emission tick did
    // NOT consume the filtered one-shot — a follow-up matching tick
    // amplifies.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add_one_shot_filtered(3.0, SourceId::from("protocol:burnout"));
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    // Tick 1 — None emission.
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
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 95.0);

    // Tick 2 — matching emission; one-shot still present, drains.
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
    // 95 - (5.0 * 3.0 = 15.0) = 80.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 80.0);
}

// ── Behavior 31: filterless persistent + filtered persistent multiply ONLY when emission matches the filter ──

#[test]
fn filterless_and_filtered_persistent_combine_only_when_filter_matches() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:global"), 2.0); // filterless
        s.add_filtered(
            SourceId::from("src:burnout"),
            3.0,
            SourceId::from("protocol:burnout"),
        );
        app.world_mut().spawn(s).id()
    };
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0))).id();

    // Tick A — matching emission, both entries amplify.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        1.0,
            source:        Some(SourceId::from("protocol:burnout")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    // 1.0 * 2.0 * 3.0 = 6.0 damage; 100 - 6 = 94.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 94.0);

    // Tick B — non-matching emission, only filterless amplifies.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        1.0,
            source:        Some(SourceId::from("protocol:debt_collector")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    // 1.0 * 2.0 = 2.0 damage; 94 - 2 = 92.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 92.0);
}

#[test]
fn filterless_and_filtered_persistent_with_none_emission_only_filterless_applies() {
    // Edge case for Behavior 31: None emission. Only the filterless
    // entry applies — filtered entry does NOT match None.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let dealer = {
        let mut s = DamageBoostStack::default();
        s.add(SourceId::from("src:global"), 2.0);
        s.add_filtered(
            SourceId::from("src:burnout"),
            3.0,
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
            amount:        1.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);
    // 1.0 * 2.0 = 2.0 damage; 100 - 2 = 98.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 98.0);
}
