use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    cells::components::{CellHeight, CellWidth},
    prelude::*,
};

// Behavior 20 — ghost cell carries the full component suite.
#[test]
fn spawn_ghosts_ghost_cell_carries_full_component_suite() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::new(100.0, 50.0),
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(
        &Cell,
        &GhostCell,
        &Position2D,
        &Scale2D,
        &Aabb2D,
        &CollisionLayers,
        &CellWidth,
        &CellHeight,
        &Hp,
        &KilledBy,
    )>();
    assert_eq!(query.iter(app.world()).count(), 1);
}

// Behavior 21 — ghost CellWidth / CellHeight pinned to GHOST_* constants.
#[test]
fn spawn_ghosts_ghost_cell_width_and_height_are_seventy_and_twenty_four() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app
        .world_mut()
        .query::<(&GhostCell, &CellWidth, &CellHeight)>();
    let dims: Vec<(f32, f32)> = query
        .iter(app.world())
        .map(|(_, cw, ch)| (cw.value, ch.value))
        .collect();
    assert_eq!(dims.len(), 1);
    assert!(
        (dims[0].0 - 70.0).abs() < f32::EPSILON,
        "width = {}",
        dims[0].0
    );
    assert!(
        (dims[0].1 - 24.0).abs() < f32::EPSILON,
        "height = {}",
        dims[0].1
    );
}

// Behavior 22 — ghost Scale2D matches dimensions.
#[test]
fn spawn_ghosts_ghost_scale_matches_dimensions() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &Scale2D)>();
    let scales: Vec<Scale2D> = query.iter(app.world()).map(|(_, s)| *s).collect();
    assert_eq!(scales.len(), 1);
    assert!(
        (scales[0].x - 70.0).abs() < f32::EPSILON,
        "x = {}",
        scales[0].x
    );
    assert!(
        (scales[0].y - 24.0).abs() < f32::EPSILON,
        "y = {}",
        scales[0].y
    );
}

// Behavior 23 — ghost Aabb2D equals half-dimensions centered at Vec2::ZERO.
#[test]
fn spawn_ghosts_ghost_aabb_is_half_dimensions_centered() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &Aabb2D)>();
    let aabbs: Vec<Aabb2D> = query.iter(app.world()).map(|(_, a)| *a).collect();
    assert_eq!(aabbs.len(), 1);
    assert_eq!(aabbs[0], Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)));
}

// Behavior 24 — ghost CollisionLayers == (CELL_LAYER, BOLT_LAYER).
#[test]
fn spawn_ghosts_ghost_collision_layers_are_cell_x_bolt() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &CollisionLayers)>();
    let layers: Vec<CollisionLayers> = query.iter(app.world()).map(|(_, l)| *l).collect();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0], CollisionLayers::new(CELL_LAYER, BOLT_LAYER));
}

// Behavior 25 — ghost Hp.current == Hp.starting; max.is_none().
#[test]
fn spawn_ghosts_ghost_hp_current_equals_starting() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 3);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app.world_mut().query::<(&GhostCell, &Hp)>();
    let ghosts: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(ghosts.len(), 1);
    let hp = ghosts[0].1;
    assert!((hp.current - 4.0).abs() < 1e-5);
    assert!((hp.starting - 4.0).abs() < 1e-5);
    assert!(hp.max.is_none());
}

// Behavior 26 — ghost carries CleanupOnExit<NodeState> at stack 1.
// Forward-compat pin: Cell carries #[require(Spatial2D,
// CleanupOnExit<NodeState>)], so ghosts inherit it automatically. This
// regression-guards against anyone silently removing the #[require].
#[test]
fn spawn_ghosts_ghost_carries_cleanup_on_exit_node_state_at_stack_one() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app
        .world_mut()
        .query::<(&GhostCell, &CleanupOnExit<NodeState>)>();
    assert_eq!(query.iter(app.world()).count(), 1);
}

// Behavior 27 — ghost carries CleanupOnExit<NodeState> at stack 3.
#[test]
fn spawn_ghosts_ghost_carries_cleanup_on_exit_node_state_at_stack_three() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 3);
    app.world_mut().spawn(PendingGhost {
        position: Vec2::ZERO,
        timer:    0.05,
    });

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut query = app
        .world_mut()
        .query::<(&GhostCell, &CleanupOnExit<NodeState>)>();
    assert_eq!(query.iter(app.world()).count(), 1);
}

// Behavior 27b — PendingGhost carries CleanupOnExit<NodeState>
// explicitly. PendingGhost is a standalone marker entity — it does
// NOT carry `Cell`, so it does NOT inherit cleanup via `#[require]`.
// The track system must attach `CleanupOnExit::<NodeState>::default()`
// to each spawn, or in-flight pendings leak when `NodeState` exits.
#[test]
fn track_deaths_pending_ghost_carries_cleanup_on_exit_node_state() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, echo_cells_track_deaths);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&PendingGhost, &CleanupOnExit<NodeState>)>();
    assert_eq!(
        query.iter(app.world()).count(),
        1,
        "PendingGhost must carry CleanupOnExit<NodeState> — it does not\
         inherit it via Cell's #[require] because it does not carry Cell"
    );
}
