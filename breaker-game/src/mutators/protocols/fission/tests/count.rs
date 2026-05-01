//! Group B — counter increment (no split threshold reached) + harness-safe
//! guards (Behaviors 6-9, 30, 31).
//!
//! Pins that `FissionCounter.kills` increments by 1 per `Destroyed<Cell>`
//! message; multi-message ticks accumulate; quiet ticks leave state
//! untouched; missing `FissionConfig` / `FissionCounter` does not panic and
//! the reader is drained so buffered messages cannot leak.

use bevy::prelude::*;

use super::{
    super::system::{FissionConfig, FissionCounter},
    helpers::{
        build_fission_app, build_fission_app_no_config, build_fission_app_no_counter,
        build_fission_app_no_registry, install_fission_config, install_fission_counter,
        seed_active_protocols_with_fission, write_destroyed_cell, write_n_destroyed_cell,
    },
};
use crate::{bolt::registry::BoltRegistry, prelude::*};

// ── Behavior 6 — counter increments by 1 on a single Destroyed<Cell> ────────

#[test]
fn counter_increments_by_one_on_single_destroyed_cell() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let bolt = app.world_mut().spawn(Bolt).id();
    // Spawn a dummy bolt because killer: Some(bolt) is semantically present.

    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 1 },
        "single message must produce kills=1; got {counter:?}"
    );
}

// ── Behavior 6 (edge case) — environmental kill also counts ────────────────-

#[test]
fn environmental_kill_also_counts() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 1 },
        "environmental kill (killer: None) must increment counter; got {counter:?}"
    );
}

// ── Behavior 7 — counter increments by N on N messages same tick ────────────

#[test]
fn counter_increments_by_n_on_n_messages_same_tick() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_n_destroyed_cell(&mut app, 3, Some(bolt));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 3 },
        "three messages must produce kills=3; got {counter:?}"
    );
}

// ── Behavior 7 (edge case) — mixed Some/None messages still accumulate ─────-

#[test]
fn mixed_killer_messages_accumulate_normally() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_destroyed_cell(&mut app, Some(bolt));
    write_destroyed_cell(&mut app, Some(bolt));
    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 3 },
        "mixed Some/None killers still accumulate to kills=3; got {counter:?}"
    );
}

// ── Behavior 8 — zero messages leaves counter unchanged ─────────────────────

#[test]
fn zero_messages_leaves_counter_unchanged() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 4);

    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 4 },
        "no messages must leave counter unchanged at 4; got {counter:?}"
    );
}

// ── Behavior 8 (edge case) — five quiet ticks still leaves counter at 4 ────-

#[test]
fn five_quiet_ticks_leave_counter_unchanged() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 4);

    for _ in 0..5 {
        tick(&mut app);
    }

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 4 },
        "five quiet ticks must leave counter at 4; got {counter:?}"
    );
}

// ── Behavior 9 — counter increment is monotonic across ticks ────────────────

#[test]
fn counter_increment_is_monotonic_across_ticks() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);
    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 2 },
        "two kills across two ticks must produce kills=2; got {counter:?}"
    );
}

// ── Behavior 9 (edge case) — interleaved empty ticks still produce 2 ───────-

#[test]
fn interleaved_empty_ticks_still_produce_two_kills() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);
    tick(&mut app);
    tick(&mut app);
    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 2 },
        "interleaved empty ticks must not lose kills; got {counter:?}"
    );
}

// ── Behavior 30 — missing FissionConfig is harness-safe ─────────────────────

#[test]
fn missing_fission_config_does_not_panic_or_increment_counter() {
    let mut app = build_fission_app_no_config();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);
    tick(&mut app);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "counter must stay at 7 when config is absent; got {counter:?}"
    );
    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "FissionConfig must not be side-effect-inserted"
    );
}

// ── Behavior 30 (edge case) — config added later does not retro-leak ───────-

#[test]
fn config_added_later_does_not_replay_buffered_messages() {
    let mut app = build_fission_app_no_config();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let bolt = app.world_mut().spawn(Bolt).id();

    // Tick 1 — write message with config absent. Reader must drain.
    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);
    assert_eq!(
        *app.world().resource::<FissionCounter>(),
        FissionCounter { kills: 7 },
        "counter must stay at 7 during config-absent tick"
    );

    // Tick 2 — install config; write NO new message. Buffered message from
    // tick 1 must NOT retroactively increment the counter.
    install_fission_config(
        &mut app,
        FissionConfig {
            kills_per_split:      8,
            divergence_angle_rad: 0.0,
        },
    );
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "pre-config-insert message must NOT leak into counter; got {counter:?}"
    );
}

// ── Behavior 31a — missing BoltRegistry is harness-safe ────────────────────-
//
// Same Option<Res<_>> harness-safe pattern as the FissionConfig and
// FissionCounter guards. Pinning all three keeps coverage symmetric.

#[test]
fn missing_bolt_registry_does_not_panic_or_increment_counter() {
    let mut app = build_fission_app_no_registry();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);

    // Reaching here without panic is the primary assertion.
    assert!(
        app.world().get_resource::<BoltRegistry>().is_none(),
        "BoltRegistry must not be side-effect-inserted when absent"
    );
    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 7 },
        "missing BoltRegistry must drain reader without incrementing counter; got {counter:?}"
    );
    let bolts = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts, 1,
        "no split should occur when BoltRegistry is absent; got {bolts}"
    );
}

// ── Behavior 31 — missing FissionCounter is harness-safe ────────────────────

#[test]
fn missing_fission_counter_does_not_panic() {
    let mut app = build_fission_app_no_counter();
    seed_active_protocols_with_fission(&mut app, 8);
    let bolt = app.world_mut().spawn(Bolt).id();

    write_destroyed_cell(&mut app, Some(bolt));
    tick(&mut app);

    // Reaching here without panic is the primary assertion.
    assert!(
        app.world().get_resource::<FissionCounter>().is_none(),
        "FissionCounter must not be side-effect-inserted when absent"
    );
    // No new bolt spawned when counter is absent.
    let bolts = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts, 1,
        "no split should occur when FissionCounter is absent; got {bolts}"
    );
}
