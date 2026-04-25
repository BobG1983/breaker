use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_stateflow::CleanupOnExit;

use super::{super::system::*, helpers::*};
use crate::prelude::*;

// Behavior 17 — CleanupOnExit<NodeState> forward-compatibility regression pin.
#[test]
fn debris_carries_cleanup_on_exit_node_state_at_stack_one() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&FractureDebris, &CleanupOnExit<NodeState>)>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

#[test]
fn debris_carries_cleanup_on_exit_node_state_at_stack_three() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 3);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&FractureDebris, &CleanupOnExit<NodeState>)>();
    assert_eq!(query.iter(app.world()).count(), 4);
}

// Behavior 18 — boundary-magnitude victim positions stay finite.
#[test]
fn boundary_magnitude_victim_position_produces_finite_debris() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 3);

    let victim = Vec2::new(1e6, -1e6);
    write_cell_destroyed(&mut app, victim);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 4);
    for pos in &positions {
        assert!(pos.x.is_finite(), "pos.x not finite: {pos:?}");
        assert!(pos.y.is_finite(), "pos.y not finite: {pos:?}");
    }

    let relative: Vec<Vec2> = positions.iter().map(|p| *p - victim).collect();
    let expected = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
        Vec2::new(0.0, -24.0),
    ];
    for exp in &expected {
        assert!(
            relative.iter().any(|r| approx_eq_vec2(*r, *exp, 1e-1)),
            "expected relative offset {exp:?} missing"
        );
    }
}

#[test]
fn boundary_magnitude_victim_position_sign_symmetric() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 3);

    let victim = Vec2::new(-1e6, 1e6);
    write_cell_destroyed(&mut app, victim);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 4);
    for pos in &positions {
        assert!(pos.x.is_finite());
        assert!(pos.y.is_finite());
    }

    let relative: Vec<Vec2> = positions.iter().map(|p| *p - victim).collect();
    let expected = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
        Vec2::new(0.0, -24.0),
    ];
    for exp in &expected {
        assert!(
            relative.iter().any(|r| approx_eq_vec2(*r, *exp, 1e-1)),
            "expected relative offset {exp:?} missing"
        );
    }
}

// Behavior 25 — gate off (stacks = 0) → no debris; gate reopens cleanly.
#[test]
fn register_gate_off_stacks_zero_does_not_spawn() {
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());
    // NO stacks added — gate off.

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn register_gate_reopens_when_stack_added_spawns_post_open_messages() {
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());

    // First tick: gate off, no message written.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Open the gate, THEN write the message.
    add_fracture_stacks(&mut app, 1);
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

#[test]
fn register_pregate_messages_drain_cleanly_before_gate_opens() {
    // Regression pin against accidental `.run_if` reintroduction on the
    // reader system: `fracture_on_death` now enforces its gate in-body
    // via `reader.clear()` so pre-gate `Destroyed<Cell>` messages drain
    // cleanly instead of accumulating and replaying on gate open. Shared
    // retrofit pattern with overcharge / drift / gravity_surge.
    //
    // Setup: gate starts CLOSED (no Fracture stack). Write one
    // Destroyed<Cell>. Tick (gate off → drain). Toggle gate ON without
    // writing a new message. Tick again. The pre-gate message must NOT
    // be consumed → 0 debris spawn.
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());

    // Tick 1 — gate off, pre-gate death written. Retrofit drains reader.
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    let mut q = app.world_mut().query::<&FractureDebris>();
    assert_eq!(
        q.iter(app.world()).count(),
        0,
        "gate closed → no debris this tick"
    );

    // Tick 2 — open gate, write no new message. The pre-gate message was
    // drained on tick 1; nothing remains to replay.
    add_fracture_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "gate open → pre-gate message already drained → 0 debris"
    );
}

// Behavior 26 — gate off (NodeState != Playing) → no debris.
#[test]
fn register_gate_off_state_not_playing_does_not_spawn() {
    let mut app = test_app_not_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);
    // Do NOT write message — gate is off, reader won't drain.

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn register_positive_control_state_playing_does_spawn() {
    // Paired positive control: same setup as the gate-off test except for
    // test_app_playing() vs test_app_not_playing(). Divergence in this
    // pair proves the NodeState gate is the discriminator.
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);
    write_cell_destroyed(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

// Behavior 27 — gate toggles open mid-run.
#[test]
fn register_gate_toggles_open_mid_run_processes_fresh_stacks() {
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());
    // Gate off initially (no stacks).

    // First tick: gate off, no processing.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Open the gate with stack=2, then write the message.
    add_fracture_stacks(&mut app, 2);
    write_cell_destroyed(&mut app, Vec2::new(10.0, 20.0));

    // Second tick: gate open, stack=2 → count=3.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 3);

    let victim = Vec2::new(10.0, 20.0);
    let expected = [
        victim + Vec2::new(70.0, 0.0),
        victim + Vec2::new(-70.0, 0.0),
        victim + Vec2::new(0.0, 24.0),
    ];
    for exp in &expected {
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
            "missing expected position {exp:?}"
        );
    }
}

#[test]
fn register_gate_open_third_tick_without_message_spawns_nothing_new() {
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    add_fracture_stacks(&mut app, 2);
    write_cell_destroyed(&mut app, Vec2::new(10.0, 20.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Third tick, no new message.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 3);
}
