//! Group E — Wave A integration tests for `DamageBoostStack` source filter.
//!
//! End-to-end pipeline coverage: a dealer with a populated
//! `DamageBoostStack` (filtered or filterless) emits `DamageDealt<TestT>`
//! messages with various `source` values, ticks the app, and asserts on
//! the victim's resulting `Hp`.
//!
//! Also smoke-checks the `entry_applies` re-export from the crate root.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageBoostStack, DamageDealt, Dmgable, Hp, RantzDmgAppExt, RantzDmgPlugin, SourceId,
    entry_applies,
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

// ── Behavior 26: `entry_applies` is reachable from the crate root re-export ──

#[test]
fn entry_applies_is_reachable_from_crate_root() {
    // Smoke-check that the `lib.rs` re-export
    // (`pub use source_id::{SourceId, entry_applies};`) actually
    // compiled. If the re-export is missing or visibility is wrong,
    // this test will fail to build at the RED gate.
    assert!(entry_applies(None, None));
    let filter = SourceId::from("protocol:burnout");
    let emission = SourceId::from("protocol:burnout");
    assert!(entry_applies(Some(&filter), Some(&emission)));
}

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
