//! Group J — integration: multi-wave breaker slow aggregate.

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::system::{ResonanceActiveSlows, resonance_tick_slows, resonance_wave_contact},
    helpers::{
        SpawnWaveParams, canonical_config, collect_sources_with_prefix, spawn_breaker_at,
        spawn_wave, test_app_playing_with_effects, tick_with_dt,
    },
};
use crate::{
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
};

// ── J1 — two simultaneous waves → aggregate 0.25 ───────────────────────

#[test]
fn j1_two_simultaneous_waves_produce_aggregate_quarter() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 0.0));
    let _wave_a = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 10.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );
    let _wave_b = spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 15.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
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
        .expect("stack should exist");
    let agg = stack.aggregate();
    assert!(
        (agg - 0.25).abs() < 1e-5,
        "Aggregate should be 0.5*0.5=0.25, got {agg}"
    );

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert_eq!(slows.len(), 2, "Two distinct slow entries expected");
    for entry in slows.values() {
        assert!((entry.remaining - 1.5).abs() < 1e-5);
        assert_eq!(entry.multiplier, OrderedFloat(0.5));
    }
}

// ── J2 — after effective_slow_duration → aggregate 1.0 ─────────────────

#[test]
fn j2_after_expiry_aggregate_returns_to_one() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(FixedUpdate, (resonance_wave_contact, resonance_tick_slows));
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 0.0));
    spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 10.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );
    spawn_wave(
        &mut app,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 15.0),
            speed:             200.0,
            slow_duration:     1.5,
            slow_strength:     0.5,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );

    // First tick: waves contact, slows seeded (remaining = 1.5).
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    app.update();

    // Second tick: advance 1.0s → remaining ~0.5.
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
    app.update();
    // Third tick: advance 0.6s → remaining ~-0.1 → expire.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.6));
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(
        slows.is_empty(),
        "Slows should all expire; got keys {:?}",
        slows.keys().collect::<Vec<_>>()
    );
    let sources = collect_sources_with_prefix(&app, breaker, "hazard:resonance:");
    assert!(
        sources.is_empty(),
        "All resonance stack entries should have been reversed, found {sources:?}"
    );
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("stack should exist");
    let agg = stack.aggregate();
    assert!(
        (agg - 1.0).abs() < f32::EPSILON,
        "Empty stack aggregate should be 1.0, got {agg}"
    );
}

// ── J3 — stack 3 slow stronger and longer than stack 1 ─────────────────

#[test]
fn j3_stack_three_slow_is_longer_and_stronger_than_stack_one() {
    let cfg = canonical_config();

    // Stack-1 sub-app.
    let mut app1 = test_app_playing_with_effects();
    app1.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker1 = spawn_breaker_at(&mut app1, Vec2::new(0.0, 0.0));
    let dur1 = cfg.effective_slow_duration(1);
    let str1 = cfg.effective_slow_strength(1);
    spawn_wave(
        &mut app1,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 5.0),
            speed:             200.0,
            slow_duration:     dur1,
            slow_strength:     str1,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );
    tick_with_dt(&mut app1, Duration::from_secs_f32(0.016));
    app1.update();

    let stack1 = app1
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker1)
        .expect("breaker1 stack");
    let agg1 = stack1.aggregate();
    assert!(
        (agg1 - 0.5).abs() < 1e-5,
        "Stack-1 aggregate should be 0.5, got {agg1}"
    );

    // Stack-3 sub-app.
    let mut app3 = test_app_playing_with_effects();
    app3.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Resonance);
    app3.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Resonance);
    app3.add_systems(FixedUpdate, resonance_wave_contact);
    let breaker3 = spawn_breaker_at(&mut app3, Vec2::new(0.0, 0.0));
    let dur3 = cfg.effective_slow_duration(3);
    let str3 = cfg.effective_slow_strength(3);
    let expected_mult3 = (1.0 - str3).clamp(0.1, 1.0);
    spawn_wave(
        &mut app3,
        SpawnWaveParams {
            pos:               Vec2::new(0.0, 5.0),
            speed:             200.0,
            slow_duration:     dur3,
            slow_strength:     str3,
            target_pos:        Vec2::new(0.0, 0.0),
            age:               0.0,
            max_lifetime:      10.0,
            contact_threshold: 16.0,
        },
    );
    tick_with_dt(&mut app3, Duration::from_secs_f32(0.016));
    app3.update();

    let stack3 = app3
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker3)
        .expect("breaker3 stack");
    let agg3 = stack3.aggregate();
    assert!(
        (agg3 - expected_mult3).abs() < 1e-4,
        "Stack-3 aggregate should be ~{expected_mult3}, got {agg3}"
    );

    assert!(
        agg3 < agg1,
        "stack3 aggregate must be < stack1 (stronger slow)"
    );

    let slows1 = &app1.world().resource::<ResonanceActiveSlows>().slows;
    let slows3 = &app3.world().resource::<ResonanceActiveSlows>().slows;
    let rem1 = slows1.values().next().unwrap().remaining;
    let rem3 = slows3.values().next().unwrap().remaining;
    assert!(
        rem3 > rem1,
        "stack3 duration ({rem3}) must exceed stack1 duration ({rem1})"
    );

    let mult1 = slows1.values().next().unwrap().multiplier.0;
    let mult3 = slows3.values().next().unwrap().multiplier.0;
    assert!(
        mult3 < mult1,
        "stack3 multiplier ({mult3}) must be smaller (stronger slow) than stack1 ({mult1})"
    );
}
