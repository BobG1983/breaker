//! Group F — `resonance_wave_contact` + MB3 (contact wins over expire).

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::system::{
        ResonanceActiveSlows, resonance_wave_contact, resonance_wave_expire, resonance_wave_travel,
    },
    helpers::{
        SpawnWaveParams, collect_sources_with_prefix, count_stack_entries_with_source,
        spawn_breaker_at, spawn_breaker_at_no_stack, spawn_wave, test_app_playing_with_effects,
        tick_with_dt,
    },
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

fn wave_source(wave: Entity) -> String {
    format!("hazard:resonance:wave:{}", wave.to_bits())
}

// ── F1 — wave within threshold fires slow and despawns ─────────────────

#[test]
fn f1_wave_within_threshold_fires_slow_and_despawns() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 58.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 50.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    assert!(
        app.world().get_entity(wave).is_err(),
        "Wave should be despawned on contact"
    );
    let source = wave_source(wave);
    let count = count_stack_entries_with_source(&app, breaker, &source);
    assert_eq!(
        count, 1,
        "Breaker stack should have exactly 1 entry for {source}, got {count}"
    );
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("breaker must have SpeedBoost stack");
    let entry_mult = stack
        .iter()
        .find(|(s, _)| s.0.as_ref() == source)
        .map(|(_, c)| c.multiplier);
    assert_eq!(entry_mult, Some(OrderedFloat(0.5)));

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    let entry = slows
        .get(&source)
        .expect("ResonanceActiveSlows should have an entry for this wave's source");
    assert!(
        (entry.remaining - 1.5).abs() < 1e-5,
        "remaining should be 1.5, got {}",
        entry.remaining
    );
    assert_eq!(
        entry.multiplier,
        OrderedFloat(0.5),
        "Stored multiplier should equal fired multiplier"
    );
}

// ── F2 — distance == contact_threshold triggers ────────────────────────

#[test]
fn f2_wave_at_exact_contact_threshold_triggers() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let _breaker = spawn_breaker_at(&mut app, Vec2::ZERO);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 16.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::ZERO,
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    assert!(
        app.world().get_entity(wave).is_err(),
        "Wave at distance == threshold should trigger contact (<=)"
    );
    let slows = app.world().resource::<ResonanceActiveSlows>();
    assert_eq!(slows.slows.len(), 1, "Exactly one active slow registered");
}

// ── F3 — distance > threshold does NOT fire ────────────────────────────

#[test]
fn f3_wave_outside_threshold_does_not_fire() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker = spawn_breaker_at(&mut app, Vec2::ZERO);
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 20.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::ZERO,
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    assert!(
        app.world().get_entity(wave).is_ok(),
        "Wave outside threshold should not be despawned"
    );
    let sources = collect_sources_with_prefix(&app, breaker, "hazard:resonance:");
    assert!(
        sources.is_empty(),
        "Breaker should have zero resonance-sourced stack entries, got {sources:?}"
    );
    assert!(
        app.world()
            .resource::<ResonanceActiveSlows>()
            .slows
            .is_empty(),
        "No active slow should be registered"
    );
}

// ── F4 — contact uses LIVE breaker distance ────────────────────────────

#[test]
fn f4_contact_distance_uses_live_breaker_not_target_pos() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let _breaker = spawn_breaker_at(&mut app, Vec2::new(100.0, 50.0));
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(100.0, 60.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 50.0), // target stored far from live breaker
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    assert!(
        app.world().get_entity(wave).is_err(),
        "Contact must use live breaker distance → wave should despawn"
    );
    let slows = app.world().resource::<ResonanceActiveSlows>();
    assert_eq!(slows.slows.len(), 1, "One slow entry expected");
}

// ── F5 — two contacting waves → two distinct stack entries ─────────────

#[test]
fn f5_two_simultaneous_waves_produce_two_distinct_stack_entries() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    let wave_a = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 55.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 50.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );
    let wave_b = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 60.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 50.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    assert!(app.world().get_entity(wave_a).is_err());
    assert!(app.world().get_entity(wave_b).is_err());

    let sources = collect_sources_with_prefix(&app, breaker, "hazard:resonance:wave:");
    assert_eq!(
        sources.len(),
        2,
        "Expected 2 resonance sources, got {sources:?}"
    );
    let src_a = wave_source(wave_a);
    let src_b = wave_source(wave_b);
    assert!(
        sources.contains(&src_a),
        "Missing source {src_a} in {sources:?}"
    );
    assert!(
        sources.contains(&src_b),
        "Missing source {src_b} in {sources:?}"
    );

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert_eq!(slows.len(), 2);
    for key in [&src_a, &src_b] {
        let entry = slows.get(key).expect("missing entry");
        assert!((entry.remaining - 1.5).abs() < 1e-5);
        assert_eq!(entry.multiplier, OrderedFloat(0.5));
    }
}

// ── F6 — contact without existing stack adds one via pipeline ──────────

#[test]
fn f6_contact_on_breaker_without_stack_creates_stack() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker = spawn_breaker_at_no_stack(&mut app, Vec2::new(0.0, 50.0));
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 55.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 50.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("EffectStack must be created on first fire");
    assert_eq!(stack.len(), 1, "Exactly 1 stack entry");
    let src = wave_source(wave);
    let sources = collect_sources_with_prefix(&app, breaker, "hazard:resonance:wave:");
    assert!(sources.contains(&src));
}

// ── F7 — contact chains with travel in same tick ───────────────────────

#[test]
fn f7_contact_chains_with_travel_in_same_tick() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(
        FixedUpdate,
        (resonance_wave_travel, resonance_wave_contact).chain(),
    );
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 0.0));
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 30.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    app.update();

    assert!(
        app.world().get_entity(wave).is_err(),
        "Wave should have traveled into contact range and been despawned"
    );
    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert_eq!(slows.len(), 1);
    let sources = collect_sources_with_prefix(&app, breaker, "hazard:resonance:wave:");
    assert_eq!(sources.len(), 1);
}

// ── MB3 — contact wins over expire when both trigger same tick ─────────

#[test]
fn mb3_contact_wins_over_expire_same_tick() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(
        FixedUpdate,
        (
            resonance_wave_travel,
            resonance_wave_contact,
            resonance_wave_expire,
        )
            .chain(),
    );
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 0.0));
    let wave = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 5.0),
            speed:             0.0, // no movement
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               9.99,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    tick_with_dt(&mut app, Duration::from_secs_f32(0.02));
    app.update();

    assert!(
        app.world().get_entity(wave).is_err(),
        "Wave should be despawned"
    );

    let source = wave_source(wave);
    let count = count_stack_entries_with_source(&app, breaker, &source);
    assert_eq!(
        count, 1,
        "Contact should fire BEFORE expire → stack entry present, got {count}"
    );
    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert_eq!(slows.len(), 1, "Exactly one active slow");
    let entry = slows.get(&source).expect("missing entry");
    assert!((entry.remaining - 1.5).abs() < 1e-5);
    assert_eq!(entry.multiplier, OrderedFloat(0.5));
}
