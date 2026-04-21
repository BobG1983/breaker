use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::prelude::*;

// ── echo_cells_track_deaths ───────────────────────────────────────────

#[test]
fn death_creates_pending_ghost() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().insert_resource(EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    });
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::new(100.0, 200.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].position - Vec2::new(100.0, 200.0)).length() < 1e-5);
    assert!((pendings[0].timer - 1.5).abs() < 1e-5);
}

#[test]
fn ghost_death_does_not_create_pending() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().insert_resource(EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    });
    let ghost_victim = app.world_mut().spawn((Cell, GhostCell)).id();
    write_destroyed(&mut app, ghost_victim, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn multiple_deaths_create_multiple_pendings() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().insert_resource(EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    });
    for i in 0..3 {
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::new(i as f32 * 100.0, 0.0));
    }
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 3);
}

// ── B. echo_cells_track_deaths — extended coverage ────────────────────

// Behavior 7 — no config → no pendings; reader drained.
#[test]
fn track_deaths_no_pending_when_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    // NO EchoCellsConfig inserted.
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn track_deaths_reader_drained_when_config_absent_then_config_installed_no_new_message() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::new(10.0, 20.0));

    // First tick: no config, reader drained via reader.clear().
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Install config and tick with NO new message — reader was drained.
    install_echo_cells_config(&mut app, canonical_config());
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// Behavior 8 — delay_secs == 0.0 short-circuits.
#[test]
fn track_deaths_no_pending_when_delay_secs_is_zero() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(
        &mut app,
        EchoCellsConfig {
            delay_secs:           0.0,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn track_deaths_reader_drained_when_delay_zero_then_delay_raised_no_new_message() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(
        &mut app,
        EchoCellsConfig {
            delay_secs:           0.0,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::ZERO);

    // First tick: delay is 0, reader drained.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Raise the delay; tick with NO new message.
    install_echo_cells_config(&mut app, canonical_config());
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// Behavior 9 — negative delay short-circuits (pins <= 0.0 predicate).
#[test]
fn track_deaths_no_pending_when_delay_secs_is_negative() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(
        &mut app,
        EchoCellsConfig {
            delay_secs:           -0.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// Behavior 10 — pending timer seeded from config.delay_secs exactly.
#[test]
fn track_deaths_pending_timer_equals_delay_secs_exactly() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(
        &mut app,
        EchoCellsConfig {
            delay_secs:           2.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].timer - 2.5).abs() < 1e-5);
}

// Behavior 11 — pending position matches victim_pos.
#[test]
fn track_deaths_pending_position_matches_victim_pos() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::new(-123.5, 456.25));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].position - Vec2::new(-123.5, 456.25)).length() < 1e-4);
}

#[test]
fn track_deaths_pending_position_stays_finite_at_boundary_magnitude() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let victim = app.world_mut().spawn(Cell).id();
    let big = Vec2::new(1e6, -1e6);
    write_destroyed(&mut app, victim, big);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!(pendings[0].position.x.is_finite());
    assert!(pendings[0].position.y.is_finite());
    assert!((pendings[0].position - big).length() < 1e-1);
}

// Behavior 12 — three distinct positions yield three distinct pendings.
#[test]
fn track_deaths_three_distinct_positions_yield_three_distinct_pendings() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let expected = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(0.0, 100.0),
    ];
    for pos in expected {
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, pos);
    }

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|p| p.position).collect();
    assert_eq!(positions.len(), 3);
    for exp in &expected {
        assert!(
            positions.iter().any(|p| (*p - *exp).length() < 1e-4),
            "missing expected position {exp:?}"
        );
    }
}

// Behavior 13 — despawned victim entity: no panic, creates pending.
// Known edge: a ghost whose entity has been reaped before the Destroyed
// message is read loses its recursion-filter protection (see Behavior 16).
// This test pins the "no panic + pending created" half of the contract
// using a plain Cell (not a ghost).
#[test]
fn track_deaths_does_not_panic_when_victim_entity_is_despawned() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::ZERO);
    // Despawn the victim AFTER writing the message but BEFORE the tracker reads it.
    app.world_mut().despawn(victim);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // ghosts.contains(stale_entity) returns false → PendingGhost created.
    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].position - Vec2::ZERO).length() < 1e-4);
}

// Behavior 14 — PendingGhost marker entity carries no GhostCell.
#[test]
fn track_deaths_pending_ghost_carries_no_ghost_cell_marker() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let victim = app.world_mut().spawn(Cell).id();
    write_destroyed(&mut app, victim, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut pending_without_ghost = app
        .world_mut()
        .query_filtered::<Entity, (With<PendingGhost>, Without<GhostCell>)>();
    assert_eq!(pending_without_ghost.iter(app.world()).count(), 1);
    let mut ghosts = app.world_mut().query_filtered::<Entity, With<GhostCell>>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
}

// Behavior 15 — mixed ghost + plain deaths spawn only the plain one.
#[test]
fn track_deaths_mixed_ghost_and_plain_deaths_spawns_only_plain() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let plain = app.world_mut().spawn(Cell).id();
    let ghost = app.world_mut().spawn((Cell, GhostCell)).id();
    write_destroyed(&mut app, plain, Vec2::new(10.0, 0.0));
    write_destroyed(&mut app, ghost, Vec2::new(20.0, 0.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].position - Vec2::new(10.0, 0.0)).length() < 1e-4);
}

// Behavior 16 — despawned ghost entity loses its recursion-filter protection.
// Known edge: Query::contains(despawned_entity) returns false, so the
// tracker cannot tell the victim was a GhostCell → a PendingGhost is
// created. This pins shipped behavior; see Open Question #2 in the test
// spec for the follow-up design discussion.
#[test]
fn track_deaths_despawned_ghost_entity_still_filtered_as_non_ghost() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    // Retrofit: reader system enforces gate in-body via reader.clear().
    // Tests that exercise the happy-path need the EchoCells hazard active.
    add_echo_cells_stacks(&mut app, 1);
    install_echo_cells_config(&mut app, canonical_config());
    let ghost = app.world_mut().spawn((Cell, GhostCell)).id();
    write_destroyed(&mut app, ghost, Vec2::new(42.0, 0.0));
    // Despawn the ghost BEFORE the tracker runs — the Query::contains
    // recursion filter can no longer see it.
    app.world_mut().despawn(ghost);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].position - Vec2::new(42.0, 0.0)).length() < 1e-4);
}

// ── C. echo_cells_spawn_ghosts — extended coverage (early-return branches) ─

// Behavior 17 — no config: early return leaves PendingGhost untouched.
#[test]
fn spawn_ghosts_no_spawn_when_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    // NO EchoCellsConfig.
    add_echo_cells_stacks(&mut app, 1);
    let pending = app
        .world_mut()
        .spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        })
        .id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // Pending entity preserved (unlike the hp <= 0.0 path which despawns it).
    assert!(app.world().get_entity(pending).is_ok());
    let mut query = app.world_mut().query::<&GhostCell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// Behavior 18 — zero stacks despawns pending without spawning ghost.
#[test]
fn spawn_ghosts_zero_stacks_despawns_pending_without_spawning_ghost() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    // Zero stacks → ghost_hp(0) == 0.0 → hp <= 0.0 branch, pending despawned.
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}

// Behavior 19 — zero base_hp despawns pending without spawning ghost.
#[test]
fn spawn_ghosts_zero_base_hp_despawns_pending_without_spawning_ghost() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(
        &mut app,
        EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              0.0,
            per_level_multiplier: 2.0,
        },
    );
    add_echo_cells_stacks(&mut app, 3);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}
