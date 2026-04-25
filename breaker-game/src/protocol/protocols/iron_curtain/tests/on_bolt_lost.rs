//! Group B — `iron_curtain_on_bolt_lost` wave behavior (Behaviors 5–18).
//!
//! Pins the full damage-wave semantics: origin damage = `bolt_base *
//! damage_fraction`; full damage inside `falloff_start`; linear falloff past
//! `falloff_start` via `(1.0 - falloff_distance/max_distance).clamp(0.0, 1.0)`;
//! zero-damage clamp emits NO message; per-bolt base-damage scaling via
//! `BoltBaseDamage` with `DEFAULT_BOLT_BASE_DAMAGE` fallback; `Dead` cells are
//! filtered out by the query; `Invulnerable` cells DO receive wave messages at
//! emit stage (post-W7) and are zeroed downstream by `invulnerable_filter::<Cell>`
//! in `DmgSystems::ApplyDamage` — see the `w7_*` tests in `scheduling.rs` for
//! the end-to-end contract. The missing-breaker case is harness-safe;
//! multi-BoltLost in the same frame produce independent waves; and degenerate
//! `max_distance <= 0.0` does not panic.

use bevy::prelude::*;

use super::{
    super::system::IronCurtainConfig,
    helpers::{
        amount_for_target, build_iron_curtain_app, build_iron_curtain_app_with_playfield_height,
        collected_iron_curtain_damage, install_iron_curtain_config,
        seed_active_protocols_with_iron_curtain, spawn_bolt_with_base_damage,
        spawn_bolt_without_base_damage, spawn_breaker_at, spawn_cell_at,
        spawn_cell_at_with_markers, write_bolt_lost,
    },
};
use crate::{prelude::*, protocol::definition::ProtocolKind};

fn iron_curtain_source() -> SourceId {
    SourceId::protocol(ProtocolKind::IronCurtain).build()
}

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

// ── Behavior 9 — multi-cell wave: full + falloff + zero ────────────────────-

#[test]
fn multi_cell_wave_mixes_full_damage_falloff_and_zero_damage() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // Distance 20.0 → full damage 10.0.
    let cell_close = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    // Distance 200.0 → factor (1 - 150/350) ≈ 0.5714286 → amount ≈ 5.7142857.
    let cell_mid = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    // Distance 350.0 → falloff_distance 300.0, factor 50/350 ≈ 0.142857,
    // amount ≈ 1.42857.
    let cell_edge = spawn_cell_at(&mut app, Vec2::new(0.0, 150.0));
    // Distance 400.0 → factor 0.0 → no message.
    let cell_max = spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        3,
        "expected 3 damage messages (cell_max yields zero → no message), got {}",
        msgs.len()
    );

    let a_close =
        amount_for_target(&msgs, cell_close).expect("cell_close must have a damage message");
    assert!(
        (a_close - 10.0).abs() < 1e-4,
        "cell_close amount expected 10.0, got {a_close}"
    );

    let a_mid = amount_for_target(&msgs, cell_mid).expect("cell_mid must have a damage message");
    assert!(
        (a_mid - 5.714_286).abs() < 1e-4,
        "cell_mid amount expected ≈5.7143, got {a_mid}"
    );

    let a_edge = amount_for_target(&msgs, cell_edge).expect("cell_edge must have a damage message");
    assert!(
        (a_edge - 1.428_571).abs() < 1e-4,
        "cell_edge amount expected ≈1.4286, got {a_edge}"
    );

    assert!(
        amount_for_target(&msgs, cell_max).is_none(),
        "cell_max must have NO damage message (zero-damage skip)"
    );
}

// ── Behavior 10 — .abs() makes cells below the breaker also take damage ────-

#[test]
fn cells_below_breaker_are_damaged_by_abs_symmetric_falloff() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // Cell y=-230 is 30 below breaker, abs distance = 30.0 ≤ 50.0 → full damage.
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -230.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "below-breaker cell must take damage");
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 10.0).abs() < 1e-4,
        "abs-distance ≤ falloff_start → full origin damage 10.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 11 — two BoltLost in one frame produce two independent waves ──-

#[test]
fn two_bolts_lost_same_frame_produce_two_independent_waves() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 20.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt_a);
    write_bolt_lost(&mut app, bolt_b);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 2, "two BoltLost → two waves → two messages");
    for m in &msgs {
        assert_eq!(m.target, cell);
        assert!(
            (m.amount - 10.0).abs() < 1e-4,
            "each wave must emit 10.0, got {}",
            m.amount
        );
    }
}

// ── Behavior 11 (edge case) — two bolts with different BoltBaseDamage ──────-

#[test]
fn two_bolts_lost_same_frame_with_different_base_damages_produce_two_independent_waves() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 20.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 40.0);

    write_bolt_lost(&mut app, bolt_a);
    write_bolt_lost(&mut app, bolt_b);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "two BoltLost → two messages, got {}",
        msgs.len()
    );
    let mut amounts: Vec<f32> = msgs.iter().map(|m| m.amount).collect();
    amounts.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    assert!(
        (amounts[0] - 10.0).abs() < 1e-4,
        "smaller amount expected 10.0 (= 20.0 * 0.5), got {}",
        amounts[0]
    );
    assert!(
        (amounts[1] - 20.0).abs() < 1e-4,
        "larger amount expected 20.0 (= 40.0 * 0.5), got {}",
        amounts[1]
    );
    for m in &msgs {
        assert_eq!(m.target, cell);
    }
}

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
    use crate::{prelude::SourceIdExt, protocol::definition::ProtocolKind};
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
    let expected = SourceId::protocol(ProtocolKind::IronCurtain).build();
    for m in &msgs {
        assert_eq!(
            m.source.as_ref(),
            Some(&expected),
            "every emitted IronCurtain damage must use builder source"
        );
    }
}
