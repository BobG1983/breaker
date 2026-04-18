//! Group F — cross-hazard / cross-source synergy.
//!
//! Pins the multiplicative `EffectStack<SizeBoostConfig>` aggregation
//! across Erosion + chip + protocol sources, cross-entity independence
//! with Haste (bolt-owned `SpeedBoostConfig`), and that updates to
//! Erosion never perturb non-Erosion entries.

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{
    super::system::ErosionState,
    helpers::{
        erosion_entries, install_erosion_state, spawn_breaker, spawn_breaker_with_stack,
        test_app_playing, tick_with_dt, wire_apply_width_only,
    },
};
use crate::{
    bolt::components::Bolt,
    effect_v3::{
        effects::{SizeBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
    },
};

// ── F36 — Erosion + chip aggregate multiplicatively, not additively ────

#[test]
fn erosion_plus_chip_aggregates_multiplicatively() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.80);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        "chip:heavy".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 2);
        // 1.25 * 0.80 = 1.0 — NOT 1.25 + 0.80 = 2.05.
        assert!(
            (stack.aggregate() - 1.0).abs() < 1e-5,
            "expected 1.0 (multiplicative), got {}",
            stack.aggregate()
        );
    }

    // Edge: change the chip multiplier to 2.0 via a fresh Breaker on the same app.
    let mut seed2 = EffectStack::<SizeBoostConfig>::default();
    seed2.push(
        "chip:heavy".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    let breaker2 = spawn_breaker_with_stack(&mut app, seed2);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack2 = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker2)
        .unwrap();
    // 2.0 * 0.80 = 1.60
    assert!(
        (stack2.aggregate() - 1.60).abs() < 1e-5,
        "expected 1.60, got {}",
        stack2.aggregate()
    );
}

// ── F37 — Erosion + chip + protocol: triple-source product ────────────

#[test]
fn erosion_plus_chip_plus_protocol_triple_source_product_aggregation() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.80);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        "chip:heavy".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    seed.push(
        "protocol:anchor".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 3);
        // 1.25 * 1.5 * 0.80 = 1.5
        assert!((stack.aggregate() - 1.50).abs() < 1e-5);

        // Chip and protocol entries remain bitwise exact.
        let entries = erosion_entries(stack);
        let chip = entries.iter().find(|(s, _)| s == "chip:heavy").unwrap();
        assert_eq!(chip.1.multiplier, OrderedFloat(1.25_f32));
        let protocol = entries
            .iter()
            .find(|(s, _)| s == "protocol:anchor")
            .unwrap();
        assert_eq!(protocol.1.multiplier, OrderedFloat(1.5_f32));
    }

    // Edge: a second tick leaves aggregate and non-Erosion multipliers bitwise stable.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 3);
    assert!((stack.aggregate() - 1.50).abs() < 1e-5);
    let entries = erosion_entries(stack);
    let chip = entries.iter().find(|(s, _)| s == "chip:heavy").unwrap();
    assert_eq!(chip.1.multiplier, OrderedFloat(1.25_f32));
    let protocol = entries
        .iter()
        .find(|(s, _)| s == "protocol:anchor")
        .unwrap();
    assert_eq!(protocol.1.multiplier, OrderedFloat(1.5_f32));
}

// ── F38 — mutating Erosion state between ticks updates ONLY Erosion ────

#[test]
fn mutating_erosion_state_between_ticks_updates_only_erosion_entry() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.80);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        "chip:heavy".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    app.world_mut()
        .resource_mut::<ErosionState>()
        .width_fraction = 0.50;
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 2);
        // 1.25 * 0.50 = 0.625
        assert!((stack.aggregate() - 0.625).abs() < 1e-5);

        let entries = erosion_entries(stack);
        let chip = entries.iter().find(|(s, _)| s == "chip:heavy").unwrap();
        assert_eq!(chip.1.multiplier, OrderedFloat(1.25_f32));
        let erosion = entries.iter().find(|(s, _)| s == "hazard:erosion").unwrap();
        let m = erosion.1.multiplier.into_inner();
        assert!((m - 0.50_f32).abs() < 1e-6, "got {m}");
    }

    // Edge: third tick with state = 1.0 — aggregate = 1.25 * 1.0 = 1.25, chip unchanged.
    app.world_mut()
        .resource_mut::<ErosionState>()
        .width_fraction = 1.0;
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 2);
    assert!((stack.aggregate() - 1.25).abs() < 1e-5);
    let entries = erosion_entries(stack);
    let chip = entries.iter().find(|(s, _)| s == "chip:heavy").unwrap();
    assert_eq!(chip.1.multiplier, OrderedFloat(1.25_f32));
}

// ── F39 — Haste (Bolt) and Erosion (Breaker) do NOT interfere ──────────

#[test]
fn haste_bolt_and_erosion_breaker_do_not_interfere() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.80);

    let breaker = spawn_breaker(&mut app);

    // Separate Bolt entity with its own Haste stack.
    let mut bolt_stack = EffectStack::<SpeedBoostConfig>::default();
    bolt_stack.push(
        "hazard:haste".to_owned(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.30),
        },
    );
    let bolt = app.world_mut().spawn((Bolt, bolt_stack)).id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        // Breaker received a SizeBoost stack with aggregate ≈ 0.80.
        let b_stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(b_stack.len(), 1);
        assert!((b_stack.aggregate() - 0.80).abs() < 1e-6);

        // Bolt's SpeedBoost stack untouched.
        let s_stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap();
        assert_eq!(s_stack.len(), 1);
        let entries: Vec<_> = s_stack.iter().collect();
        assert_eq!(entries[0].0, "hazard:haste");
        assert_eq!(entries[0].1.multiplier, OrderedFloat(1.30_f32));
    }

    // Edge: a Bolt with a SizeBoostConfig stack is NOT modified (no Breaker marker).
    let mut bolt_size_stack = EffectStack::<SizeBoostConfig>::default();
    bolt_size_stack.push(
        "misconfigured".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(7.0),
        },
    );
    let bolt_size = app.world_mut().spawn((Bolt, bolt_size_stack)).id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(bolt_size)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 7.0).abs() < 1e-5);
    let entries: Vec<_> = stack.iter().collect();
    assert_eq!(entries[0].0, "misconfigured");
    assert_eq!(entries[0].1.multiplier, OrderedFloat(7.0_f32));
}

// ── F40 — multi-source product: Erosion + two chips + one protocol ────

#[test]
fn multi_source_product_erosion_plus_two_chips_plus_one_protocol() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.50);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        "chip:heavy".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    seed.push(
        "chip:aegis".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.10),
        },
    );
    seed.push(
        "protocol:anchor".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(1.50),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 4);
        // 1.25 * 1.10 * 1.50 * 0.50 = 1.03125
        assert!(
            (stack.aggregate() - 1.03125).abs() < 1e-4,
            "expected 1.03125, got {}",
            stack.aggregate()
        );
        let entries = erosion_entries(stack);
        let heavy = entries.iter().find(|(s, _)| s == "chip:heavy").unwrap();
        assert_eq!(heavy.1.multiplier, OrderedFloat(1.25_f32));
        let aegis = entries.iter().find(|(s, _)| s == "chip:aegis").unwrap();
        assert_eq!(aegis.1.multiplier, OrderedFloat(1.10_f32));
        let protocol = entries
            .iter()
            .find(|(s, _)| s == "protocol:anchor")
            .unwrap();
        assert_eq!(protocol.1.multiplier, OrderedFloat(1.50_f32));
    }

    // Edge: second tick leaves aggregate and non-Erosion multipliers bitwise stable.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 4);
    assert!((stack.aggregate() - 1.03125).abs() < 1e-4);
    let entries = erosion_entries(stack);
    let heavy = entries.iter().find(|(s, _)| s == "chip:heavy").unwrap();
    assert_eq!(heavy.1.multiplier, OrderedFloat(1.25_f32));
    let aegis = entries.iter().find(|(s, _)| s == "chip:aegis").unwrap();
    assert_eq!(aegis.1.multiplier, OrderedFloat(1.10_f32));
    let protocol = entries
        .iter()
        .find(|(s, _)| s == "protocol:anchor")
        .unwrap();
    assert_eq!(protocol.1.multiplier, OrderedFloat(1.50_f32));
}

// ── F41 — multiplicative vs additive regression (sub-unity + big chip) ─

#[test]
fn subunity_erosion_with_big_chip_multiplies_cleanly_at_main_and_min_floor() {
    let mut app = test_app_playing();
    wire_apply_width_only(&mut app);
    install_erosion_state(&mut app, 0.40);

    let mut seed = EffectStack::<SizeBoostConfig>::default();
    seed.push(
        "chip:heavy".to_owned(),
        SizeBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    let breaker = spawn_breaker_with_stack(&mut app, seed);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        // 2.0 * 0.40 = 0.80 — NOT 2.0 + 0.40 = 2.40.
        assert!(
            (stack.aggregate() - 0.80).abs() < 1e-5,
            "expected 0.80, got {}",
            stack.aggregate()
        );
    }

    // Edge: state 0.35 (min floor) × chip 2.0 = 0.70.
    app.world_mut()
        .resource_mut::<ErosionState>()
        .width_fraction = 0.35;
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let stack = app
        .world()
        .get::<EffectStack<SizeBoostConfig>>(breaker)
        .unwrap();
    assert!(
        (stack.aggregate() - 0.70).abs() < 1e-5,
        "expected 0.70, got {}",
        stack.aggregate()
    );
}
