//! Group C — `siphon_on_cell_destroyed` reader behavior (Behaviors 9–21).
//!
//! Pins the reader's streak-update + `ReverseTimePenalty` emission rules:
//! first-kill-silent, subsequent-kill-emits; hard-set window on every kill;
//! multi-kill per frame fires one penalty per non-first kill; harness-safe
//! paths on missing `SiphonStreak` / `SiphonConfig`; run-if gates on
//! `ActiveProtocols` and `NodeState::Playing`; no-message no-op; no clamping
//! of `time_per_kill`.

use bevy::prelude::*;

use super::{
    super::system::{SiphonConfig, SiphonStreak},
    helpers::{
        build_siphon_app, build_siphon_app_no_config, build_siphon_app_no_streak,
        collected_reverse_time_penalties, install_siphon_config, install_siphon_streak,
        seed_active_protocols_with_siphon, write_cell_destroyed, write_n_cell_destroyed,
    },
};
use crate::prelude::*;

// ── Behavior 9 — first kill with empty streak: state set, no penalty ────────

#[test]
fn first_kill_with_empty_streak_sets_state_and_emits_no_penalty() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // SiphonStreak already default; SiphonConfig canonical.

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "first kill should set streak to (2.0, 1); got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "first kill should emit NO ReverseTimePenalty"
    );

    // Edge case: a second tick without any new Destroyed<Cell> leaves the
    // collector empty (message was not deferred).
    tick(&mut app);
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no messages in frame 2 should keep collector empty"
    );
}

// ── Behavior 10 — second kill within window: +1, hard-set, 1 penalty ────────

#[test]
fn second_kill_within_window_increments_resets_window_and_emits_one_penalty() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.5, 1);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       2,
        },
        "second kill should leave streak at (2.0, 2) — HARD SET window; got {streak:?}"
    );

    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(
        penalties.len(),
        1,
        "expected exactly one ReverseTimePenalty, got {}",
        penalties.len()
    );
    assert!(
        (penalties[0].seconds - 0.5).abs() < f32::EPSILON,
        "penalty.seconds expected 0.5, got {}",
        penalties[0].seconds
    );

    // Edge case check: HARD SET, not additive. `1.5 + 2.0 = 3.5` is the
    // bug value. Strict equality already pins this above.
}

// ── Behavior 11 — third kill: +1, hard-set, 1 penalty; saturation check ─────

#[test]
fn third_kill_on_streaking_streak_increments_and_emits_another_penalty() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.8, 2);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       3,
        },
        "third kill should produce (2.0, 3); got {streak:?}"
    );
    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(penalties.len(), 1, "expected exactly one penalty");
    assert!(
        (penalties[0].seconds - 0.5).abs() < f32::EPSILON,
        "penalty.seconds expected 0.5"
    );

    // Edge case — fifth kill from kill_count: 4 produces kill_count: 5.
    // Proves no modulus / no cap at 2.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.0, 4);
    write_cell_destroyed(&mut app);
    tick(&mut app);
    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak.kill_count, 5,
        "fifth kill should increment to 5, not wrap or cap; got {}",
        streak.kill_count
    );
    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(penalties.len(), 1, "fifth kill should still emit 1 penalty");
}

// ── Behavior 12 — hard window reset: set, not add ───────────────────────────

#[test]
fn kill_hard_sets_window_remaining_not_additive() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 0.1, 3);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert!(
        (streak.window_remaining - 2.0).abs() < f32::EPSILON,
        "window_remaining must be HARD SET to 2.0; additive bug would produce 2.1; \
         got {}",
        streak.window_remaining
    );
}

// ── Behavior 13 — kill after expired streak: fresh start, no penalty ────────

#[test]
fn kill_after_expired_streak_starts_fresh_without_penalty() {
    // Explicit expired-state precondition.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 0.0, 0);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "kill on expired streak should produce (2.0, 1); got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "first-kill-of-fresh-streak must emit zero penalties"
    );

    // Edge case: identical test with SiphonStreak constructed via default()
    // (same values, different origin) produces the same result.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // SiphonStreak::default() already installed by build_siphon_app via
    // .with_resource::<SiphonStreak>(); skip install_siphon_streak.

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "default()-origin expired streak should produce same (2.0, 1); got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "default()-origin first kill must still emit zero penalties"
    );
}

// ── Behavior 14 — three kills same frame: 2 penalties, total 1.0s ───────────

#[test]
fn three_kills_in_same_frame_emit_two_penalties() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // SiphonStreak::default() pre-installed.

    write_n_cell_destroyed(&mut app, 3);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       3,
        },
        "three kills: streak should be (2.0, 3); got {streak:?}"
    );

    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(
        penalties.len(),
        2,
        "three kills on empty streak: first silent + 2 penalties; got {}",
        penalties.len()
    );

    // Edge case: sum of `seconds` equals 1.0 (= 2 × 0.5) exactly.
    let total: f32 = penalties.iter().map(|m| m.seconds).sum();
    assert!(
        (total - 1.0).abs() < f32::EPSILON,
        "total seconds across frame must equal 1.0 (2 × 0.5); got {total}"
    );
}

// ── Behavior 15 — eight kills on an already-alive streak: 8 penalties ───────

#[test]
fn eight_kills_in_one_frame_on_alive_streak_emit_eight_penalties() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.0, 1);

    write_n_cell_destroyed(&mut app, 8);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak.kill_count, 9,
        "8 kills on kill_count: 1 should produce kill_count: 9; got {}",
        streak.kill_count
    );
    assert!(
        (streak.window_remaining - 2.0).abs() < f32::EPSILON,
        "window_remaining should be HARD SET to 2.0; got {}",
        streak.window_remaining
    );

    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(
        penalties.len(),
        8,
        "8 kills on alive streak should emit 8 penalties; got {}",
        penalties.len()
    );
    for (i, p) in penalties.iter().enumerate() {
        assert!(
            (p.seconds - 0.5).abs() < f32::EPSILON,
            "penalty[{i}].seconds expected 0.5, got {}",
            p.seconds
        );
    }
    let total: f32 = penalties.iter().map(|m| m.seconds).sum();
    assert!(
        (total - 4.0).abs() < f32::EPSILON,
        "total seconds expected 4.0 (8 × 0.5); got {total}"
    );
}

// ── Behavior 16 — gated off when Siphon not in ActiveProtocols ──────────────

#[test]
fn reader_is_gated_off_when_siphon_not_in_active_protocols() {
    let mut app = build_siphon_app();
    // Intentionally do NOT seed Siphon into ActiveProtocols.

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "streak must be unchanged (default) when Siphon is not active; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no penalties when Siphon is not active"
    );
}

// ── Behavior 17 — gated off when NodeState is not Playing ───────────────────

#[test]
fn reader_is_gated_off_when_node_state_is_not_playing() {
    // Build the app but DO NOT drive to NodeState::Playing — instead go
    // into ChipSelecting so the in_state(NodeState::Playing) run-if fails.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<crate::mutators::protocols::resources::ActiveProtocols>()
        .with_resource::<SiphonStreak>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<crate::state::run::node::messages::ReverseTimePenalty>()
        .in_state_chip_selecting()
        .build();
    app.world_mut().insert_resource(SiphonConfig {
        streak_window: 2.0,
        time_per_kill: 0.5,
    });
    super::super::system::wire(&mut app);
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "streak must be unchanged when NodeState != Playing; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no penalties when NodeState != Playing"
    );
}

// ── Behavior 18 — harness-safe: no panic when SiphonConfig is absent ────────

#[test]
fn reader_does_not_panic_when_siphon_config_absent() {
    let mut app = build_siphon_app_no_config();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    write_n_cell_destroyed(&mut app, 2);
    tick(&mut app);

    // No panic reaching here is the core assertion.
    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "streak unchanged when config absent; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no penalties when config absent"
    );
    assert!(
        app.world().get_resource::<SiphonConfig>().is_none(),
        "SiphonConfig must remain absent — system must not insert it"
    );

    // Edge case: second tick with no new messages produces no delayed
    // state change — the reader cleared in the prior tick.
    tick(&mut app);
    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "second tick must not retroactively process prior messages; got {streak:?}"
    );
}

// ── Behavior 19 — harness-safe: no panic when SiphonStreak is absent ────────

#[test]
fn reader_does_not_panic_when_siphon_streak_absent() {
    let mut app = build_siphon_app_no_streak();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    // No panic and no side-effect insert.
    assert!(
        app.world().get_resource::<SiphonStreak>().is_none(),
        "system must not side-effect-insert SiphonStreak when absent"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no penalties when streak absent"
    );

    // Edge case: second tick with no new messages does not cause duplicate
    // processing.
    tick(&mut app);
    assert!(
        app.world().get_resource::<SiphonStreak>().is_none(),
        "SiphonStreak must still be absent after second quiet tick"
    );
}

// ── Behavior 20 — no messages → reader leaves streak unchanged ──────────────

#[test]
fn reader_with_no_messages_leaves_streak_unchanged() {
    // Isolate the reader: do NOT call wire() so only a standalone copy
    // of siphon_on_cell_destroyed runs. This keeps the behavior focused on
    // the reader's empty-queue path without siphon_tick_streak interfering.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<crate::mutators::protocols::resources::ActiveProtocols>()
        .with_resource::<SiphonStreak>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<crate::state::run::node::messages::ReverseTimePenalty>()
        .build();
    app.world_mut().insert_resource(SiphonConfig {
        streak_window: 2.0,
        time_per_kill: 0.5,
    });
    app.add_systems(FixedUpdate, super::super::system::siphon_on_cell_destroyed);
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.5, 2);

    // No Destroyed<Cell> messages written.
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 1.5,
            kill_count:       2,
        },
        "empty-queue reader must leave streak at (1.5, 2); got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no messages → no penalties"
    );

    // Edge case: second quiet tick still no-op.
    tick(&mut app);
    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 1.5,
            kill_count:       2,
        },
        "second quiet tick must remain (1.5, 2); got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "second quiet tick emits no penalties"
    );
}

// ── Behavior 21 — no clamping of config.time_per_kill ───────────────────────

#[test]
fn reader_emits_raw_time_per_kill_without_clamping() {
    // Large value — 120.0 seconds — far outside the gameplay-plausible
    // range. The reader should pass it through unchanged.
    let mut app = build_siphon_app();
    install_siphon_config(
        &mut app,
        SiphonConfig {
            streak_window: 2.0,
            time_per_kill: 120.0,
        },
    );
    seed_active_protocols_with_siphon(&mut app, 2.0, 120.0);
    install_siphon_streak(&mut app, 1.0, 1);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(
        penalties.len(),
        1,
        "expected exactly one ReverseTimePenalty, got {}",
        penalties.len()
    );
    assert!(
        (penalties[0].seconds - 120.0).abs() < f32::EPSILON,
        "penalty.seconds must equal config.time_per_kill (120.0) — no clamp; got {}",
        penalties[0].seconds
    );
    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       2,
        },
        "streak should advance to (2.0, 2) regardless of time_per_kill value; got {streak:?}"
    );

    // Edge case: tiny value — 0.01 — also passes through unchanged.
    let mut app = build_siphon_app();
    install_siphon_config(
        &mut app,
        SiphonConfig {
            streak_window: 2.0,
            time_per_kill: 0.01,
        },
    );
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.01);
    install_siphon_streak(&mut app, 1.0, 1);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let penalties = collected_reverse_time_penalties(&app);
    assert_eq!(penalties.len(), 1, "tiny value: expected 1 penalty");
    assert!(
        (penalties[0].seconds - 0.01).abs() < f32::EPSILON,
        "penalty.seconds must equal tiny config.time_per_kill (0.01); got {}",
        penalties[0].seconds
    );
}
