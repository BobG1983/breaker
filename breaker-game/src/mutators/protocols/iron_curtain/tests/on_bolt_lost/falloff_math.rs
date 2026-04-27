//! Origin damage + linear-falloff math for the Iron Curtain wave
//! (Behaviors 5–8). Pins:
//!
//! - Cells inside `falloff_start` take full origin damage
//!   (`bolt_base * damage_fraction`).
//! - Cells past `falloff_start` take linearly reduced damage via
//!   `(1.0 - falloff_distance / max_distance).clamp(0.0, 1.0)`.
//! - Cells at or past `max_distance` clamp to zero and emit NO message.
//! - Origin damage scales with `damage_fraction`.

use bevy::prelude::*;

use super::{
    super::{
        super::system::IronCurtainConfig,
        helpers::{
            build_iron_curtain_app, build_iron_curtain_app_with_playfield_height,
            collected_iron_curtain_damage, install_iron_curtain_config,
            seed_active_protocols_with_iron_curtain, spawn_bolt_with_base_damage, spawn_breaker_at,
            spawn_cell_at, write_bolt_lost,
        },
    },
    iron_curtain_source,
};
use crate::prelude::*;

// ── Behavior 5 — BoltLost triggers wave with full origin damage on a close cell

#[test]
fn bolt_lost_triggers_wave_with_full_origin_damage_within_falloff_start() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    let _breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance = |(-170.0) - (-200.0)| = 30.0 ≤ 50.0 → full origin damage.
    let cell = spawn_cell_at(&mut app, Vec2::new(30.0, -170.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "expected exactly 1 Iron Curtain damage message, got {}",
        msgs.len()
    );
    let msg = &msgs[0];
    assert_eq!(msg.target, cell, "wave target must be the cell");
    assert!(msg.dealer.is_none(), "dealer must be None (no originator)");
    assert!(
        (msg.amount - 10.0).abs() < 1e-4,
        "amount expected 10.0 (= 20.0 * 0.5), got {}",
        msg.amount
    );
    assert_eq!(msg.source.as_ref(), Some(&iron_curtain_source()));
}

// ── Behavior 5 (edge case) — cell exactly at falloff_start boundary ────────-

#[test]
fn cell_at_exact_falloff_start_boundary_takes_full_origin_damage() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance = |(-150.0) - (-200.0)| = 50.0 (exactly at falloff_start).
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -150.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly 1 damage message");
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 10.0).abs() < 1e-4,
        "boundary cell must take full origin damage 10.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 6 — cells strictly beyond falloff_start take reduced damage ───-

#[test]
fn cell_beyond_falloff_start_takes_linearly_reduced_damage() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance = 200.0; falloff_distance = 150.0; max_distance = 350.0;
    // factor = 1.0 - 150.0/350.0 ≈ 0.5714286; amount ≈ 10.0 * 0.5714286.
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly 1 damage message");
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 5.714_286).abs() < 1e-4,
        "amount expected ≈5.714286, got {}",
        msgs[0].amount
    );
    assert_eq!(msgs[0].source.as_ref(), Some(&iron_curtain_source()));
}

// ── Behavior 6 (edge case) — cell one unit beyond falloff_start ────────────-

#[test]
fn cell_one_unit_beyond_falloff_start_takes_near_origin_damage() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance = 51.0; falloff_distance = 1.0; max_distance = 350.0;
    // factor = 1.0 - 1.0/350.0 ≈ 0.997143; amount ≈ 9.9714.
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -149.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 9.971_429).abs() < 1e-4,
        "amount expected ≈9.9714, got {}",
        msgs[0].amount
    );
}

// ── Behavior 7 — cell at maximum distance yields zero damage, no emission ──-

#[test]
fn cell_at_maximum_distance_emits_no_damage_message() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance = 400.0; falloff_distance = 350.0; max_distance = 350.0;
    // factor = 1.0 - 1.0 = 0.0; damage = 0.0 → no message.
    spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "cell at max distance must emit NO DamageDealt<Cell> message"
    );
}

// ── Behavior 7 (edge case) — cell beyond max distance clamps to zero ───────-

#[test]
fn cell_beyond_max_distance_clamps_factor_to_zero_and_emits_nothing() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance = 500.0; falloff_distance = 450.0; max_distance = 350.0;
    // raw factor = 1.0 - 450.0/350.0 ≈ -0.2857, clamped to 0.0.
    spawn_cell_at(&mut app, Vec2::new(0.0, 300.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "cell past max distance must clamp to zero and emit NO message"
    );
}

// ── Behavior 8 — wave origin damage scales with damage_fraction ────────────-

#[test]
fn wave_origin_damage_scales_with_damage_fraction() {
    let mut app = build_iron_curtain_app();
    // Override the canonical config installed by the builder — the
    // ActiveProtocols seed affects run-conditions only, not the config
    // resource. Install the tuning we want to test.
    install_iron_curtain_config(
        &mut app,
        IronCurtainConfig {
            damage_fraction: 0.25,
            falloff_start:   50.0,
        },
    );
    seed_active_protocols_with_iron_curtain(&mut app, 0.25, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 5.0).abs() < 1e-4,
        "amount expected 5.0 (= 20.0 * 0.25), got {}",
        msgs[0].amount
    );
}

// ── Behavior 8 (edge case) — damage_fraction = 1.0 emits full bolt base ────-

#[test]
fn damage_fraction_one_emits_full_bolt_base_damage_at_origin() {
    let mut app = build_iron_curtain_app();
    install_iron_curtain_config(
        &mut app,
        IronCurtainConfig {
            damage_fraction: 1.0,
            falloff_start:   50.0,
        },
    );
    seed_active_protocols_with_iron_curtain(&mut app, 1.0, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 20.0).abs() < 1e-4,
        "amount expected 20.0 (= 20.0 * 1.0 full bolt base damage), got {}",
        msgs[0].amount
    );
}
