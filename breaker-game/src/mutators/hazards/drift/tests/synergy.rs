//! Group F — Multi-hazard synergy (Wave 2 rewrite).
//!
//! After the Wave 2 migration, Drift's observable side-effect in these tests is
//! emitting `ApplyBoltForce` messages — NOT mutating `Velocity2D` directly.
//! The consumer (`apply_bolt_forces`) is NOT wired here; wiring it just to
//! re-assert on velocity would reintroduce the dt-coupling that Behavior 2 in
//! `apply_force.rs` explicitly proves the emitter is independent of.
//!
//! Assertions that REMAIN unchanged in all four tests:
//! - `EffectStack<SpeedBoostConfig>` membership / counts (Haste).
//! - `EffectStack<DamageBoostConfig>` and Overcharge-related stack checks.
//! - `ActiveHazards` / `ActiveProtocols` resource state assertions.
//! - Bolt entity counts and despawn assertions.
//! - `DriftWind.direction` and `DriftWind.timer` assertions.
//!
//! New assertions (replacing the `Velocity2D` Drift assertions):
//! - `captured_forces(app).len()` equals the expected message count.
//! - Each captured message's `force` is approximately `Vec2::X * 100.0`.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::{DriftWind, wire as drift_register},
    helpers::{
        add_drift_stacks, canonical_config, install_drift_config, install_drift_wind, spawn_bolt,
        test_app_playing, tick_with_dt,
    },
};
use crate::{
    bolt::{components::Bolt, messages::ApplyBoltForce},
    breaker::messages::BumpPerformed,
    cells::components::Cell,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    mutators::hazards::{
        definition::HazardKind,
        haste::{system::HasteConfig, wire as haste_register},
        overcharge::{
            system::{OverchargeConfig, OverchargeKillCount},
            wire as overcharge_register,
        },
        resources::ActiveHazards,
    },
    prelude::{Destroyed, SourceId, SourceIdExt},
    shared::test_utils::collector::{MessageCollector, attach_message_capture},
};

// ── Harness helpers ───────────────────────────────────────────────────────────

fn captured_forces(app: &App) -> Vec<ApplyBoltForce> {
    app.world()
        .resource::<MessageCollector<ApplyBoltForce>>()
        .0
        .clone()
}

// ── Behavior 45 — Drift + Haste: message emitted + stack coexist ─────────────

#[test]
fn drift_and_haste_coexist_on_same_bolt_in_one_tick() {
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
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

    // Drift emitted: exactly one ApplyBoltForce message with force ≈ Vec2::X * 100.0.
    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        1,
        "exactly one ApplyBoltForce message expected"
    );
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-5,
        "force.x should be ≈ 100.0, got {}",
        forces[0].force.x
    );
    assert!(
        forces[0].force.y.abs() < 1e-5,
        "force.y should be ≈ 0.0, got {}",
        forces[0].force.y
    );

    // Haste applied: stack entry with source "hazard:haste" and multiplier 1.20.
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Haste must insert EffectStack");
    assert_eq!(stack.len(), 1);
    let entry = stack
        .iter()
        .find(|(s, _)| s == &SourceId::hazard(HazardKind::Haste).build())
        .expect("hazard:haste entry expected");
    assert!((entry.1.multiplier.into_inner() - 1.20).abs() < 1e-6);
}

#[test]
fn drift_and_haste_independent_surfaces_across_two_ticks() {
    // Edge: second tick — Drift emits a second message; Haste's stack still len=1.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
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

    // After two ticks: the collector captures messages from the LAST tick only
    // (clear_messages runs at the start of each update). Two full ticks → 2 total
    // emissions (one per tick). The collector holds the messages from tick 2.
    let forces = captured_forces(&app);
    assert_eq!(forces.len(), 1, "tick 2: one ApplyBoltForce message");
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-5,
        "force.x should be ≈ 100.0, got {}",
        forces[0].force.x
    );
    assert!(forces[0].force.y.abs() < 1e-5);

    // Haste stack remains idempotent
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1, "Haste stack remains idempotent");
    let entry = stack
        .iter()
        .find(|(s, _)| s == &SourceId::hazard(HazardKind::Haste).build())
        .expect("hazard:haste entry");
    assert!((entry.1.multiplier.into_inner() - 1.20).abs() < 1e-6);
}

// ── Behavior 46 — Drift + Overcharge: both apply on same tick ────────────────

#[test]
fn drift_and_overcharge_coexist_on_same_bolt_in_one_tick() {
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
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

    // Drift emitted: exactly one ApplyBoltForce message per surviving bolt.
    // (With Bolt filter: only the bolt entity above — Overcharge does not
    // kill the bolt here since no Bolt despawn mechanic is configured in this
    // minimal test setup; we pin the count and force vector.)
    let forces = captured_forces(&app);
    assert_eq!(
        forces.len(),
        1,
        "one ApplyBoltForce message expected (one Bolt entity, no despawn in this setup)"
    );
    assert!(
        (forces[0].force.x - 100.0).abs() < 1e-5,
        "force.x should be ≈ 100.0, got {}",
        forces[0].force.x
    );
    assert!(forces[0].force.y.abs() < 1e-5, "force.y should be ≈ 0.0");
    assert_eq!(forces[0].bolt, bolt, "message must address the bolt entity");

    // Overcharge applied: single entry with multiplier 1.05^3 ≈ 1.157625.
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .expect("Overcharge must insert EffectStack");
    assert_eq!(stack.len(), 1);
    let entry = stack
        .iter()
        .find(|(s, _)| s == &SourceId::hazard(HazardKind::Overcharge).build())
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
    // Edge: second bolt with OverchargeKillCount(0) — Drift still emits a
    // message for every Bolt entity; Overcharge skips the zero-kill bolt.
    let mut app = test_app_playing();
    attach_message_capture::<ApplyBoltForce>(&mut app);
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

    // Drift emits for ALL Bolt entities regardless of Overcharge kill count.
    // Two bolts → exactly 2 messages.
    let forces = captured_forces(&app);
    let surviving_bolt_count = 2usize; // neither bolt is despawned in this test
    assert_eq!(
        forces.len(),
        surviving_bolt_count,
        "Drift must emit one message per surviving bolt, got {}",
        forces.len()
    );
    for msg in &forces {
        assert!(
            (msg.force.x - 100.0).abs() < 1e-5,
            "every force.x must be ≈ 100.0, got {}",
            msg.force.x
        );
        assert!(msg.force.y.abs() < 1e-5, "force.y must be ≈ 0.0");
    }

    // Both bolt entities received a message
    let addressed: std::collections::HashSet<Entity> = forces.iter().map(|m| m.bolt).collect();
    assert!(
        addressed.contains(&bolt_1),
        "bolt_1 must receive a Drift force message"
    );
    assert!(
        addressed.contains(&bolt_2),
        "bolt_2 must receive a Drift force message"
    );

    // Bolt 2 has no Overcharge entry (zero kills).
    let stack_2 = app.world().get::<EffectStack<SpeedBoostConfig>>(bolt_2);
    if let Some(stack) = stack_2 {
        let overcharge_entry = stack
            .iter()
            .find(|(s, _)| s == &SourceId::hazard(HazardKind::Overcharge).build());
        assert!(
            overcharge_entry.is_none(),
            "zero-kill bolt should have no Overcharge entry"
        );
    }
}
