//! Group E — tick-before-reader ordering (Behaviors 30–31).
//!
//! Pins that `siphon_tick_streak` runs BEFORE `siphon_on_cell_destroyed`
//! within the same fixed tick. A near-expiry streak that would expire during
//! the tick must do so FIRST, causing the same-frame kill to be treated as a
//! fresh first kill (silent). Compared to a mid-window streak which extends
//! normally.

use bevy::prelude::*;

use super::{
    super::system::SiphonStreak,
    helpers::{
        build_siphon_app, collected_increase_node_timers, install_siphon_streak,
        seed_active_protocols_with_siphon, write_cell_destroyed,
    },
};
use crate::prelude::*;

// ── Behavior 30 — tick BEFORE reader: near-expiry kill starts fresh streak ──

#[test]
fn tick_runs_before_reader_so_near_expiry_kill_starts_fresh_streak() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    // Set window_remaining to exactly one live timestep so tick expires it.
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    install_siphon_streak(&mut app, dt, 2);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    // If tick ran BEFORE reader: tick first drives window → 0, kill_count → 0;
    // then reader treats the kill as a fresh streak: (2.0, 1), silent.
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "tick-first ordering should make kill a FRESH streak: (2.0, 1); got {streak:?}"
    );
    assert!(
        collected_increase_node_timers(&app).is_empty(),
        "first kill of fresh streak must emit zero IncreaseNodeTimer; got {} timers",
        collected_increase_node_timers(&app).len()
    );
}

// ── Behavior 31 — mid-window streak survives tick + kill extends it ─────────

#[test]
fn mid_window_kill_continues_streak_with_one_penalty() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // window_remaining 1.0 is well above one timestep (~1/64s).
    install_siphon_streak(&mut app, 1.0, 2);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       3,
        },
        "mid-window kill should extend streak to (2.0, 3); got {streak:?}"
    );

    let timers = collected_increase_node_timers(&app);
    assert_eq!(
        timers.len(),
        1,
        "mid-window kill emits exactly one IncreaseNodeTimer; got {}",
        timers.len()
    );
    assert!(
        (timers[0].delta - 0.5).abs() < f32::EPSILON,
        "penalty.delta expected 0.5; got {}",
        timers[0].delta
    );
}
