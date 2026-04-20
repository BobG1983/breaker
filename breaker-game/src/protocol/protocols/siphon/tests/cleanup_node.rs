//! Group F — `siphon_cleanup_node` on node exit (Behaviors 32–35).
//!
//! Pins that `OnExit(NodeState::Playing)` resets `SiphonStreak` to default;
//! cleanup runs unconditionally (even when Siphon is NOT in
//! `ActiveProtocols`); re-entry does not re-populate; and absence of the
//! resource is harness-safe.

use bevy::prelude::*;

use super::{
    super::system::SiphonStreak,
    helpers::{
        build_siphon_app, build_siphon_app_no_streak, install_siphon_streak,
        seed_active_protocols_with_siphon, write_cell_destroyed,
    },
};
use crate::prelude::*;

// ── Behavior 32 — exit from Playing resets SiphonStreak to default ──────────

#[test]
fn exit_playing_resets_siphon_streak_to_default() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.5, 4);

    // Drive the `OnExit(NodeState::Playing)` schedule.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "cleanup must reset SiphonStreak to default on exit; got {streak:?}"
    );

    // Edge case: idempotent — exiting from default leaves it at default.
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    // SiphonStreak is already default.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "cleanup is idempotent on already-default streak; got {streak:?}"
    );
}

// ── Behavior 33 — cleanup runs even when Siphon is NOT active ───────────────

#[test]
fn cleanup_runs_even_when_siphon_not_in_active_protocols() {
    let mut app = build_siphon_app();
    // Intentionally do NOT seed Siphon into ActiveProtocols.
    install_siphon_streak(&mut app, 1.5, 4);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "cleanup must run unconditionally — even without Siphon active; got {streak:?}"
    );
}

// ── Behavior 34 — re-entry does not re-populate SiphonStreak ────────────────

#[test]
fn re_entry_to_playing_does_not_repopulate_streak() {
    let mut app = build_siphon_app();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);
    install_siphon_streak(&mut app, 1.5, 4);

    // Exit: Playing → AnimateOut.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    // Re-enter: AnimateOut → Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak::default(),
        "re-entry must NOT re-populate — streak stays at default; got {streak:?}"
    );

    // Edge case: a post-re-entry kill starts a fresh streak.
    write_cell_destroyed(&mut app);
    tick(&mut app);

    let streak = *app.world().resource::<SiphonStreak>();
    assert_eq!(
        streak,
        SiphonStreak {
            window_remaining: 2.0,
            kill_count:       1,
        },
        "post-re-entry kill should produce a fresh streak (2.0, 1); got {streak:?}"
    );
}

// ── Behavior 35 — harness-safe: no panic when SiphonStreak is absent ────────

#[test]
fn cleanup_does_not_panic_when_siphon_streak_absent() {
    let mut app = build_siphon_app_no_streak();
    seed_active_protocols_with_siphon(&mut app, 2.0, 0.5);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    // No panic → test reaches here. Cleanup must not side-effect-insert
    // the resource.
    assert!(
        app.world().get_resource::<SiphonStreak>().is_none(),
        "cleanup must not insert SiphonStreak as a side effect"
    );
}
