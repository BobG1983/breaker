use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// ── echo_cells_spawn_ghosts ───────────────────────────────────────────

#[test]
fn ghost_spawns_after_delay() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    app.world_mut().insert_resource(EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::EchoCells);
    let pending = app
        .world_mut()
        .spawn(PendingGhost {
            position: Vec2::new(50.0, 75.0),
            timer:    0.05,
        })
        .id();

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // PendingGhost is despawned.
    assert!(app.world().get_entity(pending).is_err());
    // Ghost cell spawned at the recorded position.
    let mut query = app.world_mut().query::<(&GhostCell, &Position2D, &Hp)>();
    let ghosts: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(ghosts.len(), 1);
    let (_, pos, hp) = ghosts[0];
    assert!((pos.0 - Vec2::new(50.0, 75.0)).length() < 1e-4);
    assert!((hp.current - 1.0).abs() < f32::EPSILON);
}

#[test]
fn ghost_hp_reflects_current_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    app.world_mut().insert_resource(EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    });
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::EchoCells);
    }
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &Hp)>();
    let ghosts: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(ghosts.len(), 1);
    // 1 * 2^2 = 4
    assert!((ghosts[0].1.current - 4.0).abs() < 1e-5);
}

#[test]
fn ghost_does_not_spawn_before_timer_expires() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    app.world_mut().insert_resource(EchoCellsConfig {
        delay_secs:           1.5,
        base_hp:              1.0,
        per_level_multiplier: 2.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::EchoCells);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    1.0,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<&GhostCell>();
    assert_eq!(query.iter(app.world()).count(), 0);
    let mut q = app.world_mut().query::<&PendingGhost>();
    let remaining = q.iter(app.world()).next().unwrap().timer;
    assert!((remaining - 0.9).abs() < 1e-5);
}

// Behavior 28 — multiple pendings all expire on the same frame.
#[test]
fn spawn_ghosts_multiple_pendings_all_expire_same_frame() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    let expected = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(200.0, 0.0),
    ];
    for pos in expected {
        app.world_mut().spawn(PendingGhost {
            position: pos,
            timer:    0.05,
        });
    }

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 3);
    for exp in &expected {
        assert!(
            positions.iter().any(|p| (*p - *exp).length() < 1e-4),
            "missing expected position {exp:?}"
        );
    }
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}

// Behavior 29 — timer decrements by delta_seconds.
#[test]
fn spawn_ghosts_timer_decrements_by_delta_seconds() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    1.5,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.4));

    let mut pendings = app.world_mut().query::<&PendingGhost>();
    let ps: Vec<_> = pendings.iter(app.world()).collect();
    assert_eq!(ps.len(), 1);
    assert!((ps[0].timer - 1.1).abs() < 1e-5);
    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
}

#[test]
fn spawn_ghosts_timer_decrements_on_two_consecutive_ticks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    1.5,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.4));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.4));

    let mut pendings = app.world_mut().query::<&PendingGhost>();
    let ps: Vec<_> = pendings.iter(app.world()).collect();
    assert_eq!(ps.len(), 1);
    assert!((ps[0].timer - 0.7).abs() < 1e-5);
    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
}

// Behavior 30 — timer expires exactly at the 0.0 boundary (strict > 0.0 check).
#[test]
fn spawn_ghosts_timer_expires_exactly_at_zero_boundary() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.016,
    });

    // timer -= 0.016 → 0.0 exactly; 0.0 > 0.0 is false → expiry branch fires.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 1);
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}

// Behavior 31 — overshoot expiry still spawns exactly once (no make-up loop).
#[test]
fn spawn_ghosts_overshoot_expiry_still_spawns_once() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.1,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 1);
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}

// Behavior 32 — mid-flight stack change affects the already-pending ghost's HP.
#[test]
fn spawn_ghosts_mid_flight_stack_change_affects_pending() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    1.0,
    });

    // First tick: 1.0 - 0.5 = 0.5, no ghost yet.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.5));
    {
        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
    }

    // Increase stacks from 1 → 3 mid-flight.
    add_echo_cells_stacks(&mut app, 2);

    // Second tick: 0.5 - 0.6 = -0.1 → expires; HP computed at 3 stacks.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.6));

    let mut query = app.world_mut().query::<(&GhostCell, &Hp)>();
    let ghosts: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(ghosts.len(), 1);
    // 1.0 * 2.0^2 = 4.0 (3 stacks at expiry, not the 1 stack at creation).
    assert!((ghosts[0].1.current - 4.0).abs() < 1e-5);
}

// Behavior 33 — spawned ghost's Position2D matches the PendingGhost position.
#[test]
fn spawn_ghosts_ghost_position_matches_pending_position() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::new(-777.0, 888.5),
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 1);
    assert!((positions[0] - Vec2::new(-777.0, 888.5)).length() < 1e-4);
}
