//! Group D — `burnout_on_bump` mega-bump consume (Behaviors D1–D9).
//!
//! Pins the `BumpPerformed` consumer:
//! - On a breaker with `mega_bump_charged`, inserts `BurnoutDamageBoost`
//!   with `multiplier = config.full_heat_damage_multiplier` on the named
//!   bolt, resets heat to `BurnoutHeat::default()`, and dispatches a
//!   shockwave at the breaker's `Position2D`.
//! - Unkindled bumps (`mega_bump_charged == false`) are no-ops.
//! - All `BumpGrade` variants fire the consume when charged.
//! - `bolt: None` does not consume the charge.
//! - Despawned breaker / despawned bolt tolerated (no panic).
//! - Multi-bolt isolation — only the named bolt gets the boost.
//! - Harness-safe: early return + reader drain when `BurnoutConfig` absent.

use bevy::prelude::*;

use super::{
    super::system::BurnoutHeat,
    helpers::{
        build_burnout_app, build_burnout_app_no_config, count_shockwave_sources,
        read_damage_boost_multiplier, read_heat, seed_active_protocols_with_burnout,
        set_heat_state, shockwave_position, shockwave_source_chip, shockwave_source_entities,
        spawn_bolt_with_base_damage, spawn_breaker_stationary, write_bump_performed,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── D1 — Mega-bump consume — boost, reset, shockwave ───────────────────────-

#[test]
fn mega_bump_consume_inserts_boost_resets_heat_fires_shockwave() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        Some(4.0),
        "BurnoutDamageBoost must be inserted with multiplier == full_heat_damage_multiplier"
    );
    assert_eq!(
        read_heat(&app, breaker),
        Some(BurnoutHeat::default()),
        "breaker's BurnoutHeat must reset to default after consume"
    );
    assert_eq!(
        count_shockwave_sources(&mut app),
        1,
        "exactly ONE shockwave entity must spawn on mega-bump"
    );
    let entities = shockwave_source_entities(&mut app);
    let entity = *entities.first().expect("at least one shockwave entity");
    let pos = shockwave_position(&app, entity).expect("shockwave entity should have Position2D");
    assert!(
        (pos - Vec2::new(0.0, -400.0)).length() < f32::EPSILON,
        "shockwave must spawn at breaker's position, got {pos:?}",
    );
    assert_eq!(
        shockwave_source_chip(&app, entity),
        Some("protocol:burnout:shockwave".into()),
        "shockwave's EffectSourceChip must carry BURNOUT_SHOCKWAVE_SOURCE"
    );
}

// ── D1b — Different breaker position — shockwave matches ───────────────────-

#[test]
fn mega_bump_shockwave_position_matches_different_breaker_position() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(100.0, -500.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    let entities = shockwave_source_entities(&mut app);
    let entity = *entities
        .first()
        .expect("expected at least one shockwave entity");
    let pos = shockwave_position(&app, entity).expect("Position2D should be present");
    assert!(
        (pos - Vec2::new(100.0, -500.0)).length() < f32::EPSILON,
        "shockwave must spawn at breaker's different position, got {pos:?}",
    );
}

// ── D2 — Bump without charge — no boost, no shockwave, heat unchanged ──────-

#[test]
fn bump_without_charge_is_a_no_op() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 0.5, 0.0, false);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        None,
        "no boost when mega_bump_charged == false"
    );
    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    // One tick of stationary drain at drain_duration 2.0:
    // (1/64) / 2.0 = 1/128 = 0.0078125. on_bump must NOT mutate heat; only
    // update_heat's deterministic per-tick drain should apply.
    let expected = 0.5 - (1.0 / 64.0) / 2.0;
    assert!(
        (h.heat - expected).abs() < 1e-4,
        "heat expected ≈ {expected} (0.5 minus one stationary-drain tick), got {}",
        h.heat
    );
    assert!(!h.mega_bump_charged);
    assert_eq!(
        count_shockwave_sources(&mut app),
        0,
        "no shockwave when mega_bump_charged == false"
    );
}

// ── D2b — BumpGrade::Early / Late also no-op when not charged ──────────────-

#[test]
fn early_and_late_bumps_without_charge_are_no_ops() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 0.5, 0.0, false);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Early);
    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Late);
    tick(&mut app);

    assert_eq!(read_damage_boost_multiplier(&app, bolt), None);
    assert_eq!(count_shockwave_sources(&mut app), 0);
}

// ── D3 — Mega-bump consume — grade does NOT gate (Late works) ──────────────-

#[test]
fn mega_bump_consume_fires_on_late_grade() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Late);
    tick(&mut app);

    assert_eq!(read_damage_boost_multiplier(&app, bolt), Some(4.0));
    assert_eq!(count_shockwave_sources(&mut app), 1);
}

// ── D3b — Early bump grade also consumes the charge ────────────────────────-

#[test]
fn mega_bump_consume_fires_on_early_grade() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Early);
    tick(&mut app);

    assert_eq!(read_damage_boost_multiplier(&app, bolt), Some(4.0));
    assert_eq!(count_shockwave_sources(&mut app), 1);
}

// ── D4 — bolt: None bump does NOT consume the charge ───────────────────────-

#[test]
fn bolt_none_bump_does_not_consume_charge_no_shockwave() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);

    write_bump_performed(&mut app, breaker, None, BumpGrade::Perfect);
    tick(&mut app);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.mega_bump_charged,
        "bolt: None must NOT consume the charge — got mega_bump_charged == false"
    );
    assert_eq!(
        count_shockwave_sources(&mut app),
        0,
        "no shockwave on bolt: None bump"
    );
}

// ── D5 — Multi-bolt isolation — only the named bolt gets the boost ─────────-

#[test]
fn multi_bolt_isolation_only_named_bolt_gets_boost() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt_a), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt_a),
        Some(4.0),
        "bolt_a must receive the boost"
    );
    assert_eq!(
        read_damage_boost_multiplier(&app, bolt_b),
        None,
        "bolt_b must NOT receive the boost"
    );
}

// ── D6 — Despawned breaker — no panic, no boost, no shockwave ──────────────-

#[test]
fn despawned_breaker_is_tolerated_no_panic() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut().entity_mut(breaker).despawn();

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert_eq!(read_damage_boost_multiplier(&app, bolt), None);
    assert_eq!(count_shockwave_sources(&mut app), 0);
}

// ── D7 — Despawned bolt — no panic; heat reset; shockwave still fires ──────-

#[test]
fn despawned_bolt_is_tolerated_shockwave_still_fires() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut().entity_mut(bolt).despawn();

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < f32::EPSILON,
        "breaker heat must reset to 0.0 on consume, got {}",
        h.heat
    );
    assert!(!h.mega_bump_charged);
    assert_eq!(
        count_shockwave_sources(&mut app),
        1,
        "shockwave must still fire even when the bolt is despawned"
    );
}

// ── D8 — Harness-safe — early-return + reader drain when config absent ─────-

#[test]
fn on_bump_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // tick 1 — config absent

    assert_eq!(read_damage_boost_multiplier(&app, bolt), None);
    assert_eq!(count_shockwave_sources(&mut app), 0);
    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.mega_bump_charged,
        "heat must be unchanged — system early-returned"
    );
}

// ── D8b — Second quiet tick — buffered message must NOT retro-apply ────────-

#[test]
fn on_bump_buffered_message_does_not_retro_apply_when_config_absent() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // tick 1 — config absent, drain reader
    tick(&mut app); // tick 2 — quiet

    assert_eq!(read_damage_boost_multiplier(&app, bolt), None);
    assert_eq!(count_shockwave_sources(&mut app), 0);
}

// ── D9 — Breaker lacks BurnoutHeat — tolerated, no consume ─────────────────-

#[test]
fn breaker_without_burnout_heat_is_tolerated_no_consume() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    // Spawn a breaker with NO BurnoutHeat component (stationary, so
    // burnout_update_heat's lazy-insert path sets default = mega_bump_charged false).
    let breaker = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -400.0)),
            Velocity2D(Vec2::ZERO),
        ))
        .id();
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app); // must not panic

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        None,
        "breaker without BurnoutHeat must not produce a boost"
    );
    assert_eq!(
        count_shockwave_sources(&mut app),
        0,
        "no shockwave when breaker has no BurnoutHeat"
    );
}
