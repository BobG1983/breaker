//! Group C — split threshold and counter reset (Behaviors 10-13).
//!
//! Pins that the Nth kill resets the counter to 0 and spawns one new bolt;
//! non-Nth kills do not split; same-frame multi-kill crossings split exactly
//! the number of times the threshold is crossed.

use bevy::prelude::*;

use super::super::{
    super::system::{FissionConfig, FissionCounter},
    helpers::{
        build_fission_app, count_bolts, install_fission_config, install_fission_counter,
        seed_active_protocols_with_fission, spawn_bolt_at_with_velocity, write_destroyed_cell,
        write_n_destroyed_cell,
    },
};
use crate::prelude::*;

// ── Behavior 10 — Nth kill resets counter to 0 and spawns one new bolt ──────

#[test]
fn nth_kill_resets_counter_to_zero_and_spawns_one_new_bolt() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));
    assert_eq!(count_bolts(&mut app), 1, "precondition: one parent bolt");

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "Nth kill must reset counter to 0; got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        2,
        "exactly one new bolt must spawn at threshold"
    );
}

// ── Behavior 10 (edge case) — kills_per_split: 1 splits every kill ─────────-

#[test]
fn kills_per_split_of_one_splits_on_every_kill() {
    let mut app = build_fission_app();
    install_fission_config(
        &mut app,
        FissionConfig {
            kills_per_split:      1,
            divergence_angle_rad: 0.0,
        },
    );
    seed_active_protocols_with_fission(&mut app, 1);
    install_fission_counter(&mut app, 0);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "every-kill-split must leave counter at 0; got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        2,
        "single kill with kills_per_split: 1 must spawn one new bolt"
    );
}

// ── Behavior 11 — non-Nth kill does NOT trigger a split ─────────────────────

#[test]
fn non_nth_kill_does_not_trigger_split() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 6);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "non-Nth kill must produce counter=7; got {counter:?}"
    );
    assert_eq!(count_bolts(&mut app), 1, "no new bolt at non-Nth kill");
}

// ── Behavior 11 (edge case) — first kill from 0 does not split at N=8 ──────-

#[test]
fn first_kill_from_zero_does_not_split_when_kills_per_split_is_eight() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    // FissionCounter defaults to { kills: 0 }.
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 1 },
        "first kill from 0 must produce counter=1; got {counter:?}"
    );
    assert_eq!(count_bolts(&mut app), 1, "no split at first kill");
}

// ── Behavior 12 — same-frame multi-kill crossing threshold splits once ──────

#[test]
fn same_frame_multi_kill_crossing_threshold_splits_exactly_once() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 6);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    // 6 + 3 messages = 9 total → cross threshold once.
    write_n_destroyed_cell(&mut app, 3, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 1 },
        "6 + 3 kills must wrap once through 8 → counter=1; got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        2,
        "exactly one new bolt in this frame"
    );
}

// ── Behavior 12 (edge case) — same-frame multi-kill crosses threshold twice -

#[test]
fn same_frame_multi_kill_crossing_threshold_twice_splits_twice() {
    let mut app = build_fission_app();
    install_fission_config(
        &mut app,
        FissionConfig {
            kills_per_split:      2,
            divergence_angle_rad: 0.0,
        },
    );
    seed_active_protocols_with_fission(&mut app, 2);
    install_fission_counter(&mut app, 7);
    let _parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    // 7 + 3 messages with kills_per_split=2 → two splits.
    // 7 → 8 → split → 0 → 1 → 2 → split → 0.
    write_n_destroyed_cell(&mut app, 3, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "two threshold crossings must leave counter at 0; got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        3,
        "two splits from one parent in one frame must leave 3 bolts total"
    );
}

// ── Behavior 13 — multi-kill below threshold never splits ───────────────────

#[test]
fn multi_kill_below_threshold_never_splits() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 2);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_n_destroyed_cell(&mut app, 3, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 5 },
        "3 kills from 2 must produce counter=5; got {counter:?}"
    );
    assert_eq!(count_bolts(&mut app), 1, "no split below threshold");
}

// ── Behavior 13 (edge case) — exactly 7 kills from 0, no split ─────────────-

#[test]
fn exactly_seven_kills_from_zero_with_threshold_eight_does_not_split() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_n_destroyed_cell(&mut app, 7, Some(parent));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "7 kills from 0 must produce counter=7 (no wrap); got {counter:?}"
    );
    assert_eq!(
        count_bolts(&mut app),
        1,
        "no split — one short of threshold"
    );
}
