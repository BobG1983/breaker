//! Group F — Multi-hazard synergy (light).
//!
//! Drift's sole observable side-effect is mutating `Velocity2D.0`. Haste
//! and Overcharge operate on `EffectStack<SpeedBoostConfig>` (consumed by
//! the effect system downstream). These tests pin that Drift's
//! `Velocity2D` mutation and the other hazards' stack entries coexist
//! without interfering.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::{DriftWind, register as drift_register},
    helpers::{
        add_drift_stacks, canonical_config, install_drift_config, install_drift_wind, spawn_bolt,
        test_app_playing, tick_with_dt,
    },
};
use crate::{
    bolt::components::Bolt,
    breaker::messages::BumpPerformed,
    cells::components::Cell,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{
        definition::HazardKind,
        hazards::{
            haste::{register as haste_register, system::HasteConfig},
            overcharge::{
                register as overcharge_register,
                system::{OverchargeConfig, OverchargeKillCount},
            },
        },
        resources::ActiveHazards,
    },
    shared::death_pipeline::Destroyed,
};

// ── Behavior 45 — Drift + Haste: direct velocity + stack coexist ─────────

#[test]
fn drift_and_haste_coexist_on_same_bolt_in_one_tick() {
    let mut app = test_app_playing();
    haste_register(&mut app);
    drift_register(&mut app);
    app.world_mut().insert_resource(HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    });
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Haste);
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Drift applied: velocity gained +100 in X.
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 100.0).abs() < 1e-3,
        "Drift should apply +100 in X, got {}",
        velocity.0.x
    );
    assert!(velocity.0.y.abs() < 1e-5);

    // Haste applied: stack entry with source "hazard:haste" and multiplier 1.20.
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Haste must insert EffectStack");
    assert_eq!(stack.len(), 1);
    let entry = stack
        .iter()
        .find(|(s, _)| s == "hazard:haste")
        .expect("hazard:haste entry expected");
    assert!((entry.1.multiplier.into_inner() - 1.20).abs() < 1e-6);
}

#[test]
fn drift_and_haste_independent_surfaces_across_two_ticks() {
    // Edge: second tick — Drift's velocity doubles, Haste's stack still len=1.
    let mut app = test_app_playing();
    haste_register(&mut app);
    drift_register(&mut app);
    app.world_mut().insert_resource(HasteConfig {
        base_percent:      20.0,
        per_level_percent: 10.0,
    });
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Haste);
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));
    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 200.0).abs() < 1e-3,
        "Drift should accumulate to 200 over 2 ticks, got {}",
        velocity.0.x
    );
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1, "Haste stack remains idempotent");
    let entry = stack
        .iter()
        .find(|(s, _)| s == "hazard:haste")
        .expect("hazard:haste entry");
    assert!((entry.1.multiplier.into_inner() - 1.20).abs() < 1e-6);
}

// ── Behavior 46 — Drift + Overcharge: both apply on same tick ────────────

#[test]
fn drift_and_overcharge_coexist_on_same_bolt_in_one_tick() {
    let mut app = test_app_playing();
    // Overcharge reads `Destroyed<Cell>` and `BumpPerformed` — message
    // channels must be initialized even if no messages are sent.
    app.add_message::<Destroyed<Cell>>();
    app.add_message::<BumpPerformed>();
    overcharge_register(&mut app);
    drift_register(&mut app);
    app.world_mut().insert_resource(OverchargeConfig {
        base_frac:      0.05,
        per_level_frac: 0.03,
    });
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Overcharge);
    add_drift_stacks(&mut app, 1);
    let bolt = app
        .world_mut()
        .spawn((Bolt, Velocity2D(Vec2::ZERO), OverchargeKillCount(3)))
        .id();

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Drift applied: +100 in X.
    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 100.0).abs() < 1e-3,
        "Drift should apply +100 in X, got {}",
        velocity.0.x
    );

    // Overcharge applied: single entry with multiplier 1.05^3 ≈ 1.157625.
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Overcharge must insert EffectStack");
    assert_eq!(stack.len(), 1);
    let entry = stack
        .iter()
        .find(|(s, _)| s == "hazard:overcharge")
        .expect("hazard:overcharge entry");
    let expected = 1.05_f32.powi(3);
    assert!(
        (entry.1.multiplier.into_inner() - expected).abs() < 1e-4,
        "expected multiplier ≈ {}, got {:?}",
        expected,
        entry.1.multiplier
    );
}

#[test]
fn drift_applies_uniformly_across_bolts_independent_of_overcharge_kills() {
    // Edge: second bolt with OverchargeKillCount(0) — Drift still applies
    // uniformly; Overcharge skips the zero-kill bolt.
    let mut app = test_app_playing();
    // Overcharge reads `Destroyed<Cell>` and `BumpPerformed` — message
    // channels must be initialized even if no messages are sent.
    app.add_message::<Destroyed<Cell>>();
    app.add_message::<BumpPerformed>();
    overcharge_register(&mut app);
    drift_register(&mut app);
    app.world_mut().insert_resource(OverchargeConfig {
        base_frac:      0.05,
        per_level_frac: 0.03,
    });
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Overcharge);
    add_drift_stacks(&mut app, 1);
    let bolt_1 = app
        .world_mut()
        .spawn((Bolt, Velocity2D(Vec2::ZERO), OverchargeKillCount(3)))
        .id();
    let bolt_2 = app
        .world_mut()
        .spawn((Bolt, Velocity2D(Vec2::ZERO), OverchargeKillCount(0)))
        .id();

    tick_with_dt(&mut app, Duration::from_secs(1));

    // Both bolts gain +100 in X from Drift.
    let vel_1 = app.world().get::<Velocity2D>(bolt_1).unwrap();
    let vel_2 = app.world().get::<Velocity2D>(bolt_2).unwrap();
    assert!((vel_1.0.x - 100.0).abs() < 1e-3);
    assert!((vel_2.0.x - 100.0).abs() < 1e-3);

    // Bolt 2 has no Overcharge entry (zero kills).
    let stack_2 = app.world().get::<EffectStack<SpeedBoostConfig>>(bolt_2);
    if let Some(stack) = stack_2 {
        let overcharge_entry = stack.iter().find(|(s, _)| s == "hazard:overcharge");
        assert!(
            overcharge_entry.is_none(),
            "zero-kill bolt should have no Overcharge entry"
        );
    }
}
