use std::time::Duration;

use bevy::prelude::*;

use super::super::{super::system::*, helpers::*};
use crate::prelude::*;

// ── D. wire(app) — full chained pipeline ──────────────────────────

// Behavior 34 — chained pipeline: track phase → spawn phase across ticks.
#[test]
fn register_chains_track_then_spawn() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(50.0, 50.0));

    // Tick 1 — tracker materializes PendingGhost { timer: 1.5, pos: (50, 50) }.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    {
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        let ps: Vec<_> = pendings.iter(app.world()).collect();
        assert_eq!(ps.len(), 1);
        assert!((ps[0].position - Vec2::new(50.0, 50.0)).length() < 1e-4);
    }

    // Tick 2 — timer 1.5 - 1.5 = 0.0; 0.0 > 0.0 is false → ghost fires.
    tick_with_dt(&mut app, Duration::from_secs_f32(1.5));
    {
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }
    let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 1);
    assert!((positions[0] - Vec2::new(50.0, 50.0)).length() < 1e-4);
}

// Behavior 34a — strict > 0.0 boundary pin via full wire() pipeline.
// Pin strict > 0.0 boundary. timer arrives at exactly 0.0 in the second
// tick's spawn system; 0.0 > 0.0 is false → ghost fires. If
// echo_cells_spawn_ghosts ever changes the comparison to >= 0.0, this
// test fails and flags the regression.
#[test]
fn register_ghost_fires_at_exactly_zero_timer_strict_gt_check() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(50.0, 50.0));

    // First tick — track phase materializes PendingGhost { timer: 1.5 }.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    // Second tick — spawn phase decrements 1.5 - 1.5 = 0.0; strict > 0.0 → false.
    tick_with_dt(&mut app, Duration::from_secs_f32(1.5));

    let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 1);
    assert!((positions[0] - Vec2::new(50.0, 50.0)).length() < 1e-4);
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}

// Behavior 34b — multi-ghost queue-and-fire test OMITTED.
// The test spec (section D, #34b) marks this as optional and
// authorizes deletion if Bevy 0.18 .chain() auto-apply-deferred
// semantics make the expected outcome non-deterministic. Single-ghost
// coverage in #34 and #34a pins the pipeline and strict-boundary
// behavior without dependence on cross-system Commands flushing.

// Behavior 35 — gate off (NodeState != Playing) → nothing tracked.
#[test]
fn register_gate_off_not_playing_does_not_track() {
    let mut app = test_app_not_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
}

// Behavior 36 — positive control: identical setup except for Playing state.
// Divergence between #35 and #36 proves the NodeState gate is the discriminator.
#[test]
fn register_positive_control_state_playing_tracks() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut pendings = app.world_mut().query::<&PendingGhost>();
    let ps: Vec<_> = pendings.iter(app.world()).collect();
    assert_eq!(ps.len(), 1);
    assert!((ps[0].position - Vec2::ZERO).length() < 1e-4);
}

// Behavior 37 — gate off (zero stacks) → nothing tracked / spawned.
#[test]
fn register_gate_off_zero_stacks_does_not_track() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());
    // NO stacks added — hazard_active(EchoCells) returns false.
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 0);
}

// Behavior 38 — pre-gate messages drain cleanly before the gate opens.
#[test]
fn register_pregate_messages_drain_cleanly_before_gate_opens() {
    // Regression pin against accidental `.run_if` reintroduction on the
    // reader system: `echo_cells_track_deaths` now enforces its gate
    // in-body via `reader.clear()` so pre-gate `Destroyed<Cell>` messages
    // drain cleanly instead of accumulating and replaying on gate open.
    // Shared retrofit pattern with fracture / overcharge / drift /
    // gravity_surge.
    //
    // Setup: gate starts CLOSED (no Echo Cells stack). Write one
    // Destroyed<Cell>. Tick (gate off → drain). Toggle gate ON without
    // writing a new message. Tick again. The pre-gate message must NOT
    // be consumed → 0 PendingGhost.
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());

    // Tick 1 — gate off, pre-gate death written. Retrofit drains reader.
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(5.0, 5.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let mut q = app.world_mut().query::<&PendingGhost>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "gate closed → no pending this tick"
        );
    }

    // Tick 2 — open gate, write no new message. The pre-gate message
    // was drained on tick 1; nothing remains to replay.
    add_echo_cells_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "gate open → pre-gate message already drained → 0 pending"
    );
}

// Behavior 39 — gate reopens when a stack is added; new message processed.
#[test]
fn register_gate_reopens_when_stack_added_processes_new_messages() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());

    // Tick 1 — gate off, no message written.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Open the gate, THEN write the message.
    add_echo_cells_stacks(&mut app, 1);
    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&PendingGhost>();
    let pendings: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(pendings.len(), 1);
    assert!((pendings[0].position - Vec2::ZERO).length() < 1e-4);
}

// Behavior 40 — gate reopens when NodeState enters Playing.
// TODO(recovery): needs in-test NodeState transition helper — see
// TestAppBuilder::in_state_node_playing pattern at
// breaker-game/src/shared/test_utils/builder.rs:80-107. Skipped per
// test spec Open Question #1; covered indirectly by the stack-add gate
// test (Behavior 39) and the positive-control pair (Behaviors 35/36).

// Behavior 41 — second tick without a new message does not spawn more.
#[test]
fn register_second_tick_without_message_does_not_spawn_more() {
    let mut app = test_app_playing();
    wire(&mut app);
    install_echo_cells_config(&mut app, canonical_config());
    add_echo_cells_stacks(&mut app, 1);

    write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(50.0, 50.0));
    // Tick 1 — track phase materializes pending.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    // Tick 2 — timer 1.5 - 1.5 = 0.0 → ghost fires.
    tick_with_dt(&mut app, Duration::from_secs_f32(1.5));

    // Sanity: one ghost, zero pendings after materialization.
    {
        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 1);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Two more ticks with NO new message → no additional ghosts.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut ghosts = app.world_mut().query::<&GhostCell>();
    assert_eq!(ghosts.iter(app.world()).count(), 1);
    let mut pendings = app.world_mut().query::<&PendingGhost>();
    assert_eq!(pendings.iter(app.world()).count(), 0);
}
