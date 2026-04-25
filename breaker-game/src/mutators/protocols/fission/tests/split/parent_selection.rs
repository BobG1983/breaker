//! Group E — parent selection under `Destroyed<Cell>.killer` semantics
//! (Behaviors 21-23).
//!
//! Pins environmental (`killer: None`) → first-active-bolt selection;
//! `killer: Some(bolt)` → that specific bolt; despawned-killer fallback to
//! first-in-query; and no-bolts-at-split-time still resets the counter.

use bevy::prelude::*;

use super::super::{
    super::system::FissionCounter,
    helpers::{
        bolt_pos_and_vel, bolts_other_than, build_fission_app, count_bolts,
        install_fission_counter, seed_active_protocols_with_fission, spawn_bolt_at_with_velocity,
        write_destroyed_cell,
    },
};
use crate::prelude::*;

// ── Behavior 21 — environmental kill splits first active bolt ───────────────

#[test]
fn environmental_kill_splits_first_activebolt() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent =
        spawn_bolt_at_with_velocity(&mut app, Vec2::new(100.0, 50.0), Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "counter must reset to 0 on environmental kill crossing threshold"
    );
    assert_eq!(
        count_bolts(&mut app),
        2,
        "one new bolt from environmental kill"
    );

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let (pos, _vel) = bolt_pos_and_vel(&app, others[0]);
    assert_eq!(
        pos,
        Vec2::new(100.0, 50.0),
        "environmental kill must split the sole active bolt"
    );
}

// ── Behavior 21 (edge case) — two bolts, environmental splits one of them ──-

#[test]
fn environmental_kill_with_twobolts_splits_exactly_one() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let a = spawn_bolt_at_with_velocity(&mut app, Vec2::new(100.0, 50.0), Vec2::new(0.0, 400.0));
    let b = spawn_bolt_at_with_velocity(&mut app, Vec2::new(-100.0, 50.0), Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    assert_eq!(
        count_bolts(&mut app),
        3,
        "exactly one new bolt from environmental kill with two parents"
    );
    // The new bolt's position must match ONE of the two parents' positions
    // (query iteration order is not guaranteed across Bevy releases).
    let pairs: Vec<(Entity, Vec2)> = app
        .world_mut()
        .query_filtered::<(Entity, &Position2D), With<Bolt>>()
        .iter(app.world())
        .map(|(e, p)| (e, p.0))
        .collect();
    let new_positions: Vec<Vec2> = pairs
        .into_iter()
        .filter(|(e, _)| *e != a && *e != b)
        .map(|(_, p)| p)
        .collect();
    assert_eq!(new_positions.len(), 1, "exactly one new bolt position");
    let p = new_positions[0];
    assert!(
        p == Vec2::new(100.0, 50.0) || p == Vec2::new(-100.0, 50.0),
        "new bolt position must match one of the two parents; got {p:?}"
    );
}

// ── Behavior 22 — killer: Some(parent) targets that specific bolt ───────────

#[test]
fn killer_some_targets_the_specified_bolt() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let a = spawn_bolt_at_with_velocity(&mut app, Vec2::new(100.0, 50.0), Vec2::new(0.0, 400.0));
    let b = spawn_bolt_at_with_velocity(&mut app, Vec2::new(-100.0, 50.0), Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(a));
    tick(&mut app);

    assert_eq!(count_bolts(&mut app), 3, "exactly one new bolt");
    // The new bolt must be at A's position (100.0, 50.0), not B's.
    let pairs: Vec<(Entity, Vec2)> = app
        .world_mut()
        .query_filtered::<(Entity, &Position2D), With<Bolt>>()
        .iter(app.world())
        .map(|(e, p)| (e, p.0))
        .collect();
    let new_positions: Vec<Vec2> = pairs
        .into_iter()
        .filter(|(e, _)| *e != a && *e != b)
        .map(|(_, p)| p)
        .collect();
    assert_eq!(new_positions.len(), 1);
    assert_eq!(
        new_positions[0],
        Vec2::new(100.0, 50.0),
        "new bolt must be at A's position, not B's; got {:?}",
        new_positions[0]
    );
}

// ── Behavior 22 (edge case) — killer: Some(despawned) falls back ───────────-

#[test]
fn killer_some_despawned_entity_fallsback_to_firstbolt_without_panic() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let live = spawn_bolt_at_with_velocity(&mut app, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));

    // Spawn a bolt, then despawn it so its Entity id is dangling.
    let deadbolt =
        spawn_bolt_at_with_velocity(&mut app, Vec2::new(50.0, 50.0), Vec2::new(0.0, 400.0));
    app.world_mut().despawn(deadbolt);

    write_destroyed_cell(&mut app, Some(deadbolt));
    tick(&mut app);

    // Reaching here means no panic. Counter reset and one new bolt exist.
    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "counter must reset regardless of despawned killer"
    );
    // `live` still present plus exactly one new bolt.
    let others = bolts_other_than(&mut app, live);
    assert_eq!(
        others.len(),
        1,
        "exactly one new bolt spawned via first-in-query fallback; got {}",
        others.len()
    );
}

// ── Behavior 23 — no bolts at split time: counter resets, no new bolt ───────

#[test]
fn nobolts_in_world_still_resets_counter_no_spawn() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    // NO bolts spawned.

    write_destroyed_cell(&mut app, None);
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "counter must reset even when no bolt exists to split"
    );
    assert_eq!(
        count_bolts(&mut app),
        0,
        "no new bolt when world is bolt-empty"
    );
}

// ── Behavior 23 (edge case) — despawned killer + no other bolts, no panic ──-

#[test]
fn despawned_killer_with_no_otherbolts_resets_counter_without_panic() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    let deadbolt =
        spawn_bolt_at_with_velocity(&mut app, Vec2::new(0.0, 0.0), Vec2::new(0.0, 400.0));
    app.world_mut().despawn(deadbolt);

    write_destroyed_cell(&mut app, Some(deadbolt));
    tick(&mut app);

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter { kills: 0 },
        "counter must reset to 0 even with no live bolts"
    );
    assert_eq!(
        count_bolts(&mut app),
        0,
        "no new bolt when no live bolt exists to split"
    );
}
