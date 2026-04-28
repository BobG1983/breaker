//! Wave B — Full-pipeline integration tests for `VulnerableStack`'s
//! source-filter behavior. Behaviors 16–18 of the Wave B test spec.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_dmg::{
    DamageDealt, Destroyed, Dmgable, Hp, RantzDmgAppExt, RantzDmgPlugin, SourceId, VulnerableStack,
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

// ── Behavior 16: filtered persistent vulnerability — matching emission IS
//    amplified ──

#[test]
fn filtered_persistent_vulnerability_matching_emission_is_amplified() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0), stack)).id();
    let dealer = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("hazard:diffusion")),
            _marker:       PhantomData,
        });
    tick(&mut app);

    // 5.0 base × 2.0 vulnerable amplification = 10.0 damage; 100 - 10 = 90.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 90.0);
    let destroyed_count = app
        .world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .count();
    assert_eq!(destroyed_count, 0);
    assert!(app.world().get_entity(victim).is_ok());
}

#[test]
fn filtered_persistent_vulnerability_none_emission_does_not_amplify() {
    // Edge case for Behavior 16: a follow-up emission with `source: None`
    // is NOT amplified by the filtered entry.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0), stack)).id();
    let dealer = app.world_mut().spawn_empty().id();

    // Tick 1: matching emission amplifies (HP 100 → 90).
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("hazard:diffusion")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 90.0);

    // Tick 2: `source: None` is filtered out — only base damage applies.
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
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 85.0);
}

// ── Behavior 17: filtered persistent vulnerability — non-matching emission
//    is NOT amplified ──

#[test]
fn filtered_persistent_vulnerability_non_matching_emission_is_not_amplified() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0), stack)).id();
    let dealer = app.world_mut().spawn_empty().id();

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

    // 5.0 × 1.0 (filter rejected) = 5.0 damage; HP 100 - 5 = 95.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 95.0);
}

#[test]
fn mixed_filtered_and_filterless_persistent_with_non_matching_emission_uses_filterless_only() {
    // Edge case for Behavior 17: a stack with one filtered entry (filter
    // "hazard:diffusion", multiplier 2.0) and one filterless entry
    // (multiplier 1.5). A non-matching emission yields 5.0 × 1.5 = 7.5.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let mut stack = VulnerableStack::default();
    stack.add_filtered(
        SourceId::from("mark:fragility"),
        2.0,
        SourceId::from("hazard:diffusion"),
    );
    stack.add(SourceId::from("mark:bare"), 1.5);
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0), stack)).id();
    let dealer = app.world_mut().spawn_empty().id();

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

    // 5.0 × 1.5 = 7.5 damage; HP 100 - 7.5 = 92.5.
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 92.5);
}

// ── Behavior 18: filtered one-shot vulnerability — non-matching preserves,
//    matching consumes ──

#[test]
fn filtered_one_shot_vulnerability_non_matching_preserves_then_matching_consumes() {
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(3.0, SourceId::from("hazard:diffusion"));
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0), stack)).id();
    let dealer = app.world_mut().spawn_empty().id();

    // Tick 1: non-matching emission. Filter rejects; HP drops by base 5.0.
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
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 95.0);
    // The one-shot must be preserved on the lane.
    assert!(
        !app.world()
            .get::<VulnerableStack>(victim)
            .unwrap()
            .is_empty()
    );

    // Tick 2: matching emission consumes the one-shot — 5.0 × 3.0 = 15.0.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("hazard:diffusion")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 80.0);
    assert!(
        app.world()
            .get::<VulnerableStack>(victim)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn filtered_one_shot_vulnerability_drains_after_matching_consume() {
    // Edge case for Behavior 18: a third tick with a matching source
    // (post-drain) drops HP by base only — proves the one-shot truly
    // drained, not reapplied.
    let mut app = app_with_plugin();
    let _ = app.register_dmgable::<TestT>();

    let mut stack = VulnerableStack::default();
    stack.add_one_shot_filtered(3.0, SourceId::from("hazard:diffusion"));
    let victim = app.world_mut().spawn((TestT, Hp::new(100.0), stack)).id();
    let dealer = app.world_mut().spawn_empty().id();

    // Tick 1: non-matching — preserved. HP 100 → 95.
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

    // Tick 2: matching — consumes, HP 95 → 80.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("hazard:diffusion")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 80.0);

    // Tick 3: matching but stack is empty — only base damage. HP 80 → 75.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(DamageDealt::<TestT> {
            dealer:        Some(dealer),
            attributed_to: None,
            target:        victim,
            amount:        5.0,
            source:        Some(SourceId::from("hazard:diffusion")),
            _marker:       PhantomData,
        });
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 75.0);
}
