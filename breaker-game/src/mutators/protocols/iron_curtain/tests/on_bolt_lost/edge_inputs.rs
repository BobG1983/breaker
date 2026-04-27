//! Edge-input behavior for the Iron Curtain wave (Behaviors 12–18, B30).
//! Pins:
//!
//! - No `BoltLost` → no wave fires.
//! - Missing `BoltBaseDamage` (component absent or bolt despawned)
//!   falls back to `DEFAULT_BOLT_BASE_DAMAGE`.
//! - `Dead` cells are filtered out by the query; all-dead → 0 messages.
//! - Post-W7: `Invulnerable` cells DO receive a wave message at emit
//!   time (raw amount); the pipeline's `invulnerable_filter::<Cell>`
//!   zeroes downstream — see `scheduling.rs::w7_*` for the end-to-end
//!   contract. The combined `Dead + Invulnerable` case is excluded by
//!   the `Dead` filter alone.
//! - No alive cells / no breaker → harness-safe; second tick does not
//!   leak a prior `BoltLost`.
//! - Degenerate `max_distance <= 0.0` does not panic or emit NaN.
//! - `B30`: emitted `source` matches the builder-produced
//!   `protocol:iron_curtain` `SourceId`.

use bevy::prelude::*;

use super::{
    super::{
        super::system::IronCurtainConfig,
        helpers::{
            amount_for_target, build_iron_curtain_app,
            build_iron_curtain_app_with_playfield_height, collected_iron_curtain_damage,
            install_iron_curtain_config, seed_active_protocols_with_iron_curtain,
            spawn_bolt_with_base_damage, spawn_bolt_without_base_damage, spawn_breaker_at,
            spawn_cell_at, spawn_cell_at_with_markers, write_bolt_lost,
        },
    },
    iron_curtain_source,
};
use crate::prelude::*;

// ── Behavior 12 — no BoltLost → no wave fires ──────────────────────────────-

#[test]
fn no_bolt_lost_messages_means_no_waves_fire() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let _bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no BoltLost → no Iron Curtain damage messages"
    );
}

// ── Behavior 13 — BoltBaseDamage absent → DEFAULT_BOLT_BASE_DAMAGE fallback -

#[test]
fn missing_bolt_base_damage_uses_default_fallback() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_without_base_damage(&mut app);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "one wave expected");
    // DEFAULT_BOLT_BASE_DAMAGE = 10.0, damage_fraction = 0.5 → amount 5.0.
    assert!(
        (msgs[0].amount - 5.0).abs() < 1e-4,
        "fallback amount expected 5.0 (= 10.0 * 0.5), got {}",
        msgs[0].amount
    );
}

// ── Behavior 13 (edge case) — bolt despawned before system runs ────────────-

#[test]
fn bolt_despawned_before_system_uses_default_base_damage_fallback() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);
    app.world_mut().despawn(bolt);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "one wave expected even after bolt despawn");
    // DEFAULT_BOLT_BASE_DAMAGE = 10.0, damage_fraction = 0.5 → amount 5.0.
    assert!(
        (msgs[0].amount - 5.0).abs() < 1e-4,
        "fallback amount expected 5.0 (= DEFAULT 10.0 * 0.5), got {}",
        msgs[0].amount
    );
}

// ── Behavior 14 — Dead cells are filtered out ──────────────────────────────-

#[test]
fn dead_cells_receive_no_wave_damage() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell_alive = spawn_cell_at_with_markers(&mut app, Vec2::new(0.0, -180.0), false, false);
    let cell_dead = spawn_cell_at_with_markers(&mut app, Vec2::new(10.0, -180.0), true, false);
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "only alive cell takes damage");
    assert_eq!(msgs[0].target, cell_alive);
    assert!(
        msgs.iter().all(|m| m.target != cell_dead),
        "dead cell must not receive a damage message"
    );
}

// ── Behavior 14 (edge case) — all cells Dead → zero messages ───────────────-

#[test]
fn all_cells_dead_produces_zero_damage_messages() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at_with_markers(&mut app, Vec2::new(0.0, -180.0), true, false);
    spawn_cell_at_with_markers(&mut app, Vec2::new(10.0, -180.0), true, false);
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "all-dead cell set must produce zero messages"
    );
}

// ── Behavior 15 (W7) — Invulnerable cells DO receive a wave message at emit time.
//    `LiveCellQuery` no longer filters invulnerable cells; the pipeline's
//    `invulnerable_filter::<Cell>` (in DmgSystems::ApplyDamage) zeroes `amount`
//    downstream. THIS harness is emitter-only (`build_iron_curtain_app` does
//    NOT install the pipeline filter), so the captured amount is the raw emit
//    amount — NOT zero. See scheduling.rs's `w7_*` tests for the end-to-end
//    pipeline-zeroed contract.

#[test]
fn invulnerable_cells_emit_wave_messages_at_emit_stage_pipeline_zeros_downstream() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell_open = spawn_cell_at_with_markers(&mut app, Vec2::new(0.0, -180.0), false, false);
    let cell_locked = spawn_cell_at_with_markers(&mut app, Vec2::new(10.0, -180.0), false, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    // Post-W7: both cells receive messages at emit stage. The invulnerable
    // cell's message carries the RAW emitted amount (no filter runs in this
    // harness — the filter lives in the full pipeline).
    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "both cells receive an Iron Curtain message at emit stage, got {}",
        msgs.len()
    );

    let a_open = amount_for_target(&msgs, cell_open)
        .expect("non-invulnerable cell must have a damage message");
    assert!(
        (a_open - 10.0).abs() < 1e-4,
        "non-invulnerable cell raw emit amount expected 10.0 (= 20.0 * 0.5), got {a_open}"
    );

    let a_locked = amount_for_target(&msgs, cell_locked)
        .expect("invulnerable cell must also have a (raw, pre-pipeline) message");
    assert!(
        (a_locked - 10.0).abs() < 1e-4,
        "invulnerable cell raw emit amount expected 10.0 (pipeline zeroes \
         downstream in DmgSystems::ApplyDamage — not in this emitter-only harness), \
         got {a_locked}"
    );
}

// ── Behavior 15 (W7 edge case) — cell with both Dead and Invulnerable is
//    excluded by the `Dead` filter alone. Post-W7, `Invulnerable` no longer
//    excludes — this test now pins `Dead` exclusion for the combined-marker
//    case.

#[test]
fn cell_with_dead_and_invulnerable_is_excluded_by_dead_filter() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell_open = spawn_cell_at_with_markers(&mut app, Vec2::new(0.0, -180.0), false, false);
    let cell_both = spawn_cell_at_with_markers(&mut app, Vec2::new(10.0, -180.0), true, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    // Post-W7: `Without<Invulnerable>` no longer applies, but `Without<Dead>`
    // still does. The Dead+Invulnerable cell is excluded by `Dead` alone.
    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "only the open cell receives a message");
    assert_eq!(msgs[0].target, cell_open);
    assert!(
        msgs.iter().all(|m| m.target != cell_both),
        "Dead+Invulnerable cell must be excluded by the Dead filter"
    );
}

// ── Behavior 16 — no alive cells → wave fires silently ─────────────────────-

#[test]
fn no_alive_cells_produces_zero_messages_and_no_panic() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // No cells spawned at all.
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app); // must not panic

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no cells → no messages"
    );
}

// ── Behavior 17 — breaker query empty → wave is skipped silently ───────────-

#[test]
fn breaker_query_empty_wave_is_skipped_silently() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    // NO Breaker entity spawned.
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app); // must not panic

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no breaker → no wave, no panic"
    );
}

// ── Behavior 17 (edge case) — subsequent tick produces no leaked message ───-

#[test]
fn breaker_query_empty_subsequent_tick_does_not_leak_prior_message() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    // NO Breaker entity spawned.
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);
    // Second tick with no further writes — must stay empty (no leaked wave).
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "subsequent tick must not resurface the earlier BoltLost as a wave"
    );
}

// ── Behavior 18 — degenerate max_distance does not panic or emit NaN ───────-

#[test]
fn degenerate_max_distance_does_not_panic_or_emit_nan_or_negative() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    install_iron_curtain_config(
        &mut app,
        IronCurtainConfig {
            damage_fraction: 0.5,
            falloff_start:   600.0,
        },
    );
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 600.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // distance 200.0 < falloff_start 600.0 → within full-damage zone.
    let cell_within = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    // distance 700.0 > falloff_start 600.0 → else branch with max_distance = -200.0.
    // quotient (100.0 / -200.0) = -0.5, .clamp(0.0, 1.0) = 0.0 → factor 1.0 - 0.0 = 1.0
    // → full damage 10.0 (design doc applies .clamp to the quotient).
    let cell_beyond = spawn_cell_at(&mut app, Vec2::new(0.0, 500.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app); // must not panic

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "both cells must receive full origin damage under degenerate tuning"
    );
    let a_within = amount_for_target(&msgs, cell_within).expect("cell_within must have a message");
    assert!(
        (a_within - 10.0).abs() < 1e-4,
        "within-origin cell amount expected 10.0, got {a_within}"
    );
    let a_beyond = amount_for_target(&msgs, cell_beyond).expect("cell_beyond must have a message");
    assert!(
        (a_beyond - 10.0).abs() < 1e-4,
        "degenerate-clamp cell amount expected 10.0, got {a_beyond}"
    );
    for m in &msgs {
        assert!(
            m.amount.is_finite() && m.amount > 0.0,
            "amount must be finite positive, got {}",
            m.amount
        );
    }
}

// ── B30: source matches builder-produced protocol:iron_curtain ──

#[test]
fn iron_curtain_wave_damage_source_equals_builder() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(30.0, -170.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);
    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert!(
        !msgs.is_empty(),
        "expected at least one wave damage message"
    );
    let expected = iron_curtain_source();
    for m in &msgs {
        assert_eq!(
            m.source.as_ref(),
            Some(&expected),
            "every emitted IronCurtain damage must use builder source"
        );
    }
}
