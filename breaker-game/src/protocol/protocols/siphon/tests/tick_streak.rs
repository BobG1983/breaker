//! Group D — `siphon_tick_streak` countdown (Behaviors 22–29).
//!
//! Pins the tick-side semantics: decrement by `time.delta_secs()`, reset both
//! fields on expiry (`<= 0.0` branch), no negative clamp, idempotence on
//! already-expired streaks, harness-safe paths on missing resources, and
//! run-if gates (`protocol_active(Siphon)` + `in_state(NodeState::Playing)`).
//!
//! Timestep is read dynamically from `Time<Fixed>::timestep()` to stay
//! decoupled from the Bevy default (1/64s as of 0.18).

use bevy::prelude::*;

use super::{
    super::system::{SiphonConfig, SiphonStreak},
    helpers::{
        build_siphon_app, build_siphon_app_no_config, build_siphon_app_no_streak,
        collected_reverse_time_penalties, install_siphon_streak, seed_active_protocols_with_siphon,
    },
};
use crate::prelude::*;

/// Live `Time<Fixed>` timestep in seconds.
fn live_timestep_secs(app: &App) -> f32 {
    app.world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32()
}

// ── Behavior 22 — active streak decrements by delta_secs each tick ──────────

#[test]
fn active_streak_decrements_window_remaining_by_delta_each_tick() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 2.0, 1);

    let dt = live_timestep_secs(&app);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    let expected = 2.0 - dt;
    assert!(
        (streak.window_remaining - expected).abs() < 1e-4,
        "window_remaining expected {expected} (2.0 - {dt}), got {}",
        streak.window_remaining
    );
    assert_eq!(
        streak.kill_count, 1,
        "kill_count unchanged on non-expiring tick; expected 1, got {}",
        streak.kill_count
    );

    // Edge case: fresh app, tick exactly 5 times → 2.0 - 5.0 * dt.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 2.0, 1);

    let dt = live_timestep_secs(&app);
    for _ in 0..5 {
        tick(&mut app);
    }
    let streak = *app.world().resource::<SiphonStreak>();
    let expected = 5.0f32.mul_add(-dt, 2.0);
    assert!(
        (streak.window_remaining - expected).abs() < 1e-3,
        "after 5 ticks: window_remaining expected {expected}, got {}",
        streak.window_remaining
    );
}

// ── Behavior 23 — expiry: both fields reset to zero ────────────────────────-

#[test]
fn expiry_resets_both_fields_to_zero() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // window_remaining below the default timestep (1/64 ≈ 0.0156) so one tick
    // expires it.
    install_siphon_streak(&mut app, 0.01, 3);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 0.0,
            kill_count:       0,
        },
        "expiry must reset both fields to zero; got {streak:?}"
    );

    // Edge case: already-expired streak is idempotent across a tick.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 0.0, 0);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "already-expired streak should remain default after tick; got {streak:?}"
    );
}

// ── Behavior 24 — exact boundary (window == delta) expires ─────────────────-

#[test]
fn window_equal_to_timestep_expires_to_zero() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    let dt = live_timestep_secs(&app);
    install_siphon_streak(&mut app, dt, 2);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 0.0,
            kill_count:       0,
        },
        "window_remaining == dt should expire to (0.0, 0); got {streak:?}"
    );

    // Edge case: window_remaining slightly above the timestep leaves
    // `window_remaining` positive and `kill_count` unchanged.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    let dt = live_timestep_secs(&app);
    install_siphon_streak(&mut app, dt + 1e-7, 2);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert!(
        streak.window_remaining > 0.0,
        "window_remaining > 0 when starting value slightly exceeds timestep; got {}",
        streak.window_remaining
    );
    assert!(
        streak.window_remaining < 1e-6,
        "window_remaining should be near zero (~1e-7); got {}",
        streak.window_remaining
    );
    assert_eq!(
        streak.kill_count, 2,
        "kill_count should be unchanged when window stays positive; got {}",
        streak.kill_count
    );
}

// ── Behavior 25 — negative clamp: never leaves window < 0 ───────────────────

#[test]
fn tick_never_leaves_window_remaining_negative() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 0.001, 1);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert!(
        (streak.window_remaining - 0.0).abs() < f32::EPSILON,
        "window_remaining should clamp to 0.0, not go negative; got {}",
        streak.window_remaining
    );
    assert_eq!(
        streak.kill_count, 0,
        "kill_count should reset to 0 on expiry; got {}",
        streak.kill_count
    );

    // Edge case: window_remaining: 0.0 is idempotent.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 0.0, 0);
    tick(&mut app);
    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "already-expired streak: tick is a no-op; got {streak:?}"
    );
}

// ── Behavior 26 — no active streak + no messages = no state change ──────────

#[test]
fn tick_does_not_touch_streak_when_no_active_streak_and_no_messages() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // SiphonStreak::default() already installed via build_siphon_app.

    for _ in 0..3 {
        tick(&mut app);
        let streak = *app.world().resource::<SiphonStreak>();
        assert_eq!(
            streak,
            SiphonStreak::default(),
            "quiet idle: streak must remain default; got {streak:?}"
        );
    }
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "no penalties across 3 quiet ticks"
    );

    // Edge case: a single kill lands, then 5 idle ticks decay it naturally
    // — window_remaining shrinks by 5 * dt but never becomes negative;
    // no further penalties are emitted beyond the (first-kill silent) one.
    super::helpers::write_cell_destroyed(&mut app);
    tick(&mut app);
    let streak_after_kill = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak_after_kill.kill_count, 1,
        "first kill should set kill_count: 1; got {}",
        streak_after_kill.kill_count
    );
    let dt = live_timestep_secs(&app);
    for i in 0..5 {
        tick(&mut app);
        let streak = *app.world().resource::<SiphonStreak>();
        // After kill, window_remaining = 2.0 (tick runs BEFORE reader, so the
        // kill-frame tick is a no-op on the default streak). After `i+1` more
        // idle ticks, it should be `2.0 - (i+1)*dt`.
        // For small i this stays clearly positive.
        let expected = (i as f32 + 1.0).mul_add(-dt, 2.0);
        assert!(
            (streak.window_remaining - expected).abs() < 1e-3,
            "quiet idle tick {i}: window_remaining expected {expected}, got {}",
            streak.window_remaining
        );
    }
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "only the (silent) first kill occurred; no penalties should be emitted"
    );
}

// ── Behavior 27 — harness-safe: no panic when SiphonConfig is absent ────────

#[test]
fn tick_does_not_panic_when_siphon_config_absent() {
    let mut app = build_siphon_app_no_config();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.5, 2);

    // Three consecutive ticks: streak must remain EXACTLY (1.5, 2).
    for tick_idx in 0..3 {
        tick(&mut app);
        let streak = *app.world().resource::<SiphonStreak>();
        assert_eq!(
            streak,
            SiphonStreak {
                window_remaining: 1.5,
                kill_count:       2,
            },
            "tick {tick_idx} with no config should leave streak at (1.5, 2); got {streak:?}"
        );
    }
}

// ── Behavior 28 — harness-safe: no panic when SiphonStreak is absent ────────

#[test]
fn tick_does_not_panic_when_siphon_streak_absent() {
    let mut app = build_siphon_app_no_streak();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    tick(&mut app);

    assert!(
        app.world().get_resource::<SiphonStreak>().is_none(),
        "tick must not side-effect-insert SiphonStreak when absent"
    );
}

// ── Behavior 29 — gated off when Siphon is not active ───────────────────────

#[test]
fn tick_is_gated_off_when_siphon_not_active() {
    let mut app = build_siphon_app();
    // Intentionally do NOT seed Siphon.
    install_siphon_streak(&mut app, 2.0, 2);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       2,
        },
        "tick must not run when Siphon is not active; streak (2.0, 2) should be unchanged; \
         got {streak:?}"
    );

    // Edge case: drive to chip-selecting state (NodeState != Playing) with
    // Siphon seeded in ActiveProtocols; tick is still gated off.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<crate::protocol::resources::ActiveProtocols>()
        .with_resource::<SiphonStreak>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<crate::state::run::node::messages::ReverseTimePenalty>()
        .in_state_chip_selecting()
        .build();
    app.world_mut().insert_resource(SiphonConfig {
        streak_window: 2.0,
        time_per_kill: 0.5,
    });
    super::super::system::register(&mut app);
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 2.0, 2);

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       2,
        },
        "NodeState != Playing should gate tick off; streak (2.0, 2) unchanged; got {streak:?}"
    );
}
