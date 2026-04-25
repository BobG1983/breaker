//! Group G — `wire` wiring + run-condition gates (Behaviors 36–42).
//!
//! Pins that `wire`-wired systems run under the correct schedules, gated
//! by `protocol_active(Siphon)` + `in_state(NodeState::Playing)` on the
//! `FixedUpdate` systems, with `siphon_tick_streak` ordered BEFORE
//! `siphon_on_cell_destroyed`, and `siphon_cleanup_node` wired into
//! `OnExit(NodeState::Playing)`.

use bevy::prelude::*;

use super::{
    super::system::{SiphonConfig, SiphonStreak, wire},
    helpers::{
        build_siphon_app, collected_reverse_time_penalties, install_siphon_streak,
        seed_active_protocols_with_siphon, write_cell_destroyed,
    },
};
use crate::prelude::*;

// ── Behavior 36 — wire wires the reader under the right run-conditions ──

#[test]
fn register_wires_siphon_on_cell_destroyed_in_fixed_update() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "wire-wired reader must process first kill to (2.0, 1); got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "first kill emits zero penalties (wire-wired path)"
    );
}

// ── Behavior 37 — wire-wired reader gated off when Siphon NOT active ────

#[test]
fn register_wired_reader_is_gated_off_when_siphon_not_active() {
    let mut app = build_siphon_app();
    // Intentionally do NOT seed Siphon.

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "wire-wired reader must not run when Siphon not active; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "wire-wired reader emits no penalties when Siphon not active"
    );
}

// ── Behavior 38 — wire-wired reader gated off when NodeState != Playing ─

#[test]
fn register_wired_reader_is_gated_off_when_node_state_not_playing() {
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
    wire(&mut app);
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "wire-wired reader must be gated off when NodeState != Playing; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "wire-wired reader emits no penalties when NodeState != Playing"
    );
}

// ── Behavior 39 — wire wires tick BEFORE reader ─────────────────────────

#[test]
fn register_wires_tick_before_reader() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    install_siphon_streak(&mut app, dt, 2);

    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "with tick BEFORE reader, near-expiry kill must become fresh streak; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "fresh-streak first kill must emit zero penalties under wire ordering"
    );
}

// ── Behavior 40 — wire wires cleanup in OnExit(NodeState::Playing) ──────

#[test]
fn register_wires_cleanup_on_exit_node_state_playing() {
    let mut app = build_siphon_app();
    install_siphon_streak(&mut app, 1.5, 4);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "wire must wire cleanup in OnExit(NodeState::Playing); got {streak:?}"
    );
}

// ── Behavior 41 — wire does not panic when resources are absent ─────────

#[test]
fn register_does_not_panic_when_streak_and_config_absent() {
    // Build without SiphonStreak and without SiphonConfig.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<crate::mutators::protocols::resources::ActiveProtocols>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<crate::state::run::node::messages::ReverseTimePenalty>()
        .build();
    wire(&mut app);
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    for _ in 0..3 {
        tick(&mut app);
    }

    // No panic → reaching here is a pass. Neither resource should
    // appear spontaneously.
    assert!(
        app.world().get_resource::<SiphonStreak>().is_none(),
        "wire must not side-effect-insert SiphonStreak"
    );
    assert!(
        app.world().get_resource::<SiphonConfig>().is_none(),
        "wire must not side-effect-insert SiphonConfig"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "wire-wired systems emit no penalties when resources absent"
    );
}

// ── Behavior 42 — plugin_builds smoke: schedule ticks with no messages ──────

#[test]
fn register_schedule_ticks_with_no_messages() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // SiphonStreak::default() already installed.

    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "default streak is idle — tick is a no-op; got {streak:?}"
    );
    assert!(
        collected_reverse_time_penalties(&app).is_empty(),
        "quiet tick emits no penalties"
    );
}
