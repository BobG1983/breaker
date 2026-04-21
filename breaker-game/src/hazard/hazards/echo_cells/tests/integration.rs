use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── activate — preserved scaffold tests ────────────────────────────────

#[test]
fn activate_with_matching_tuning_inserts_config() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::EchoCells {
                delay_secs:           2.0,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
}

#[test]
fn activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            &mut commands,
        );
    });
    app.update();
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// ── D. register(app) — full chained pipeline ──────────────────────────

// Behavior 34 — chained pipeline: track phase → spawn phase across ticks.
#[test]
fn register_chains_track_then_spawn() {
    let mut app = test_app_playing();
    register(&mut app);
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

// Behavior 34a — strict > 0.0 boundary pin via full register() pipeline.
// Pin strict > 0.0 boundary. timer arrives at exactly 0.0 in the second
// tick's spawn system; 0.0 > 0.0 is false → ghost fires. If
// echo_cells_spawn_ghosts ever changes the comparison to >= 0.0, this
// test fails and flags the regression.
#[test]
fn register_ghost_fires_at_exactly_zero_timer_strict_gt_check() {
    let mut app = test_app_playing();
    register(&mut app);
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
    register(&mut app);
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
    register(&mut app);
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
    register(&mut app);
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
    register(&mut app);
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
    register(&mut app);
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
    register(&mut app);
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

// ── E. activate — extended coverage ───────────────────────────────────

// Behavior 42 — matching EchoCells tuning pins all three fields.
#[test]
fn activate_now_with_matching_tuning_pins_all_three_fields() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           3.0,
            base_hp:              7.5,
            per_level_multiplier: 1.25,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 3.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 7.5).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 1.25).abs() < f32::EPSILON);
}

// Behavior 43 — mismatched Drift tuning inserts nothing.
#[test]
fn activate_now_with_mismatched_drift_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Drift {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
    );
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 44 — mismatched Fracture tuning inserts nothing.
#[test]
fn activate_now_with_mismatched_fracture_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Fracture {
            base_splits:      2,
            per_level_splits: 1,
        },
    );
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 45 — mismatched Overcharge tuning inserts nothing.
#[test]
fn activate_now_with_mismatched_overcharge_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Overcharge {
            base_frac:      0.05,
            per_level_frac: 0.03,
        },
    );
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 46 — mismatched Cascade with NaN does not panic.
#[test]
fn activate_now_with_mismatched_cascade_nan_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::Cascade {
            base_heal:      f32::NAN,
            per_level_heal: f32::NAN,
        },
    );
    assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
}

// Behavior 47 — second activate overwrites (last-write-wins).
#[test]
fn second_activate_overwrites_echo_cells_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           3.0,
            base_hp:              5.0,
            per_level_multiplier: 1.5,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 3.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 5.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 1.5).abs() < f32::EPSILON);
}

// Behavior 48 — third activate overwrites to boundary zeros.
#[test]
fn third_activate_overwrites_to_boundary_values() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           3.0,
            base_hp:              5.0,
            per_level_multiplier: 1.5,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           0.0,
            base_hp:              0.0,
            per_level_multiplier: 0.0,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 0.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 0.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 0.0).abs() < f32::EPSILON);
}

// Behavior 49 — mismatch after match preserves the existing config.
#[test]
fn activate_now_mismatch_after_match_preserves_existing_config() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           2.0,
            base_hp:              4.0,
            per_level_multiplier: 2.0,
        },
    );
    activate_now(
        &mut app,
        &HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 4.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 2.0).abs() < f32::EPSILON);
}

// Behavior 50 — activate on an empty world does not panic.
#[test]
fn activate_now_on_empty_world_does_not_panic() {
    let mut app = TestAppBuilder::new().build();
    activate_now(
        &mut app,
        &HazardTuning::EchoCells {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        },
    );

    let cfg = app.world().resource::<EchoCellsConfig>();
    assert!((cfg.delay_secs - 1.5).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 1.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_multiplier - 2.0).abs() < f32::EPSILON);
}

// ── F. Component derive pins ──────────────────────────────────────────

// Static trait-bound assertions — compile-time pins for Clone/Copy/Default
// without triggering clippy::clone_on_copy (which fires on `.clone()` of
// Copy types). Calling the zero-body fn is a no-op at runtime; the trait
// bound is checked at monomorphization.
const fn assert_clone<T: Clone>() {}
const fn assert_copy<T: Copy>() {}
const fn assert_default<T: Default>() {}

// Behavior 51 — PendingGhost derives Copy + Clone.
#[test]
fn pending_ghost_is_clone_copy() {
    assert_clone::<PendingGhost>();
    assert_copy::<PendingGhost>();
    let a = PendingGhost {
        position: Vec2::new(1.0, 2.0),
        timer:    0.5,
    };
    let b = a; // copy, not move — a remains usable below.
    let c = a;
    assert_eq!(a.position, b.position);
    assert!((a.timer - c.timer).abs() < f32::EPSILON);
}

// Behavior 52 — GhostCell derives Default + Clone + Copy.
#[test]
fn ghost_cell_is_clone_copy_default() {
    assert_clone::<GhostCell>();
    assert_copy::<GhostCell>();
    assert_default::<GhostCell>();
    // Copy semantics pin: `a` remains usable after being copied twice.
    let a = GhostCell;
    let (b, c) = (a, a);
    let _ = b;
    let _ = c;
    // Default-constructor pin: `GhostCell::default()` returns the
    // canonical unit value (use the bare struct to avoid
    // `clippy::default_constructed_unit_structs`).
    let _: GhostCell = GhostCell;
}

// Behavior 53 — EchoCellsConfig derives Clone + Copy.
#[test]
fn echo_cells_config_is_clone_copy() {
    assert_clone::<EchoCellsConfig>();
    assert_copy::<EchoCellsConfig>();
    let cfg = canonical_config();
    let dup = cfg; // copy, not move — cfg remains usable below.
    assert!((dup.delay_secs - cfg.delay_secs).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 1.0).abs() < f32::EPSILON);
}
