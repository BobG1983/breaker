//! Group C — `spawn_gravity_wells` system.
//!
//! Every test wires only `spawn_gravity_wells` via `wire_spawn_only`. This
//! bypasses `register`-installed gates; gating is tested in Group F.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use super::{
    super::system::{GravitySurgeConfig, GravityWell},
    helpers::{
        add_gravity_surge_stacks, canonical_config, install_gravity_surge_config, test_app_playing,
        tick_with_dt, wire_spawn_only, write_cell_destroyed,
    },
};
use crate::prelude::NodeState;

// ── Behavior 16 — destroyed cell at (100,200) spawns well — PRESERVED ───

#[test]
fn destroyed_cell_spawns_well_at_position() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    app.world_mut().insert_resource(GravitySurgeConfig {
        base_duration_secs:      2.0,
        per_level_duration_secs: 1.0,
        base_strength:           500.0,
        per_level_strength_frac: 0.5,
    });
    app.world_mut()
        .resource_mut::<crate::mutators::hazards::resources::ActiveHazards>()
        .add_stack(crate::mutators::hazards::definition::HazardKind::GravitySurge);

    write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    let (well, pos) = wells[0];
    assert!((pos.0 - Vec2::new(100.0, 200.0)).length() < f32::EPSILON);
    assert!((well.remaining - 2.0).abs() < 1e-5);
    assert!((well.strength - 500.0).abs() < 1e-4);
}

#[test]
fn destroyed_cell_spawns_well_at_non_trivial_position() {
    // Edge: victim_pos passed through verbatim (not rounded/offset).
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::new(-42.5, 17.25));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    let (_well, pos) = wells[0];
    assert!((pos.0.x - (-42.5)).abs() < f32::EPSILON);
    assert!((pos.0.y - 17.25).abs() < f32::EPSILON);
}

#[test]
fn spawned_well_carries_cleanup_on_exit_node_state_marker() {
    // Regression pin — prevents well-entity leak across nodes. Matches
    // the Resonance precedent (`CleanupOnExit<NodeState>` on spawned
    // wave entities).
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::new(0.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&GravityWell, &CleanupOnExit<NodeState>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "spawned gravity well must carry CleanupOnExit<NodeState>"
    );
}

// ── Behavior 17 — stack=3 produces duration=4.0 ─────────────────────────

#[test]
fn spawn_at_stack_three_sets_duration_four_and_sqrt_scaled_strength() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 3);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    let well = wells[0];
    assert!((well.remaining - 4.0).abs() < 1e-5);
    let expected_strength = 500.0 * 0.5_f32.mul_add((2.0_f32).sqrt(), 1.0);
    assert!(
        (well.strength - expected_strength).abs() < 1e-3,
        "expected strength ≈ {expected_strength}, got {}",
        well.strength
    );
}

#[test]
fn spawn_at_stack_five_sets_duration_six_and_strength_thousand() {
    // Edge: stack=5 → duration=6, strength=1000.
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 5);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    let well = wells[0];
    assert!((well.remaining - 6.0).abs() < 1e-5);
    assert!((well.strength - 1000.0).abs() < 1e-3);
}

// ── Behavior 18 — no config → no spawn; reader cleared — PRESERVED ──────

#[test]
fn spawn_noop_without_config() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    app.world_mut()
        .resource_mut::<crate::mutators::hazards::resources::ActiveHazards>()
        .add_stack(crate::mutators::hazards::definition::HazardKind::GravitySurge);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn spawn_noop_without_config_reader_cleared_across_ticks() {
    // Edge: reader.clear() on first tick drained the message; subsequent
    // tick with config installed does not retroactively spawn.
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);

    // Install config mid-way — the prior message was drained, so tick 2
    // (with no new message) must still find zero wells.
    install_gravity_surge_config(&mut app, canonical_config());
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// ── Behavior 19 — stacks=0 → duration=0 → no spawn ──────────────────────

#[test]
fn spawn_noop_with_zero_stacks() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    // NO stacks added — stacks = 0 → duration = 0.

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn spawn_noop_with_zero_stacks_reader_cleared() {
    // Edge: adding a stack mid-way without writing a new Destroyed<Cell>
    // must still yield 0 wells (original message drained).
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Now add a stack — reader was cleared, so stale messages can't fire.
    add_gravity_surge_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// ── Behavior 20 — zero base_strength silences spawn (strength branch) ──

#[test]
fn spawn_noop_with_zero_base_strength() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(
        &mut app,
        GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           0.0,
            per_level_strength_frac: 0.5,
        },
    );
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn spawn_noop_with_zero_base_strength_at_high_stack() {
    // Edge: base=0, frac=0 — still zero at stack 5.
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(
        &mut app,
        GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           0.0,
            per_level_strength_frac: 0.0,
        },
    );
    add_gravity_surge_stacks(&mut app, 5);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// ── Behavior 21 — zero base_duration_secs silences spawn (duration branch)

#[test]
fn spawn_noop_with_zero_base_duration() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(
        &mut app,
        GravitySurgeConfig {
            base_duration_secs:      0.0,
            per_level_duration_secs: 0.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        },
    );
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn spawn_with_zero_base_duration_but_positive_per_level_spawns_at_stack_two() {
    // Edge: base=0, per_level=1 — stack 1 duration=0 (no spawn),
    // stack 2 duration=1 (spawn).
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(
        &mut app,
        GravitySurgeConfig {
            base_duration_secs:      0.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        },
    );
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 0);

    // Bump stacks to 2 and write a new message — duration=1 now.
    add_gravity_surge_stacks(&mut app, 1);
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 1);
    assert!((wells[0].remaining - 1.0).abs() < 1e-5);
}

// ── Behavior 22 — multiple Destroyed<Cell> messages → one well per ─────

#[test]
fn multiple_cell_destroyed_messages_spawn_one_well_each() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::new(10.0, 20.0));
    write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
    write_cell_destroyed(&mut app, Vec2::new(-50.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 3);

    let positions: Vec<Vec2> = wells.iter().map(|(_, p)| p.0).collect();
    let targets = [
        Vec2::new(10.0, 20.0),
        Vec2::new(100.0, 200.0),
        Vec2::new(-50.0, 0.0),
    ];
    for target in &targets {
        assert!(
            positions
                .iter()
                .any(|p| (*p - *target).length() < f32::EPSILON),
            "missing well at {target:?}, got {positions:?}"
        );
    }
    for (well, _) in &wells {
        assert!((well.remaining - 2.0).abs() < 1e-5);
        assert!((well.strength - 500.0).abs() < 1e-4);
    }
}

#[test]
fn multiple_cell_destroyed_same_position_spawn_independent_entities() {
    // Edge: duplicate positions → 5 independent entities at (0,0).
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);

    for _ in 0..5 {
        write_cell_destroyed(&mut app, Vec2::ZERO);
    }
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(Entity, &GravityWell, &Position2D)>();
    let wells: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(wells.len(), 5);
    for (_, _, pos) in &wells {
        assert!(pos.0.length() < f32::EPSILON);
    }
    // All entity ids distinct:
    let ids: Vec<Entity> = wells.iter().map(|(e, ..)| *e).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j]);
        }
    }
}

// ── Behavior 23 — wells spawn across ticks accumulate ──────────────────

#[test]
fn wells_accumulate_across_ticks() {
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);

    // Tick 1.
    write_cell_destroyed(&mut app, Vec2::new(10.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    {
        let mut q = app.world_mut().query::<&GravityWell>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    // Tick 2 — additional message, additional well.
    write_cell_destroyed(&mut app, Vec2::new(20.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

#[test]
fn wells_no_spurious_spawn_when_reader_empty() {
    // Edge: tick 3 with no new message → still 2 wells.
    let mut app = test_app_playing();
    wire_spawn_only(&mut app);
    install_gravity_surge_config(&mut app, canonical_config());
    add_gravity_surge_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::new(10.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    write_cell_destroyed(&mut app, Vec2::new(20.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    // Tick 3 — no write.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&GravityWell>();
    assert_eq!(query.iter(app.world()).count(), 2);
}
