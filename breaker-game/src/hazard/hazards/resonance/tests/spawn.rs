//! Group D — `resonance_spawn_waves` pruning, threshold, spawning.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{
        ResonanceTracker, ResonanceWave, resonance_spawn_waves, resonance_track_kills,
    },
    helpers::{
        canonical_config, spawn_breaker_at, test_app_playing, tick_n, tick_with_dt,
        write_cell_destroyed,
    },
};
use crate::{
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

fn add_stack(app: &mut App, count: u32) {
    // remove existing stack (inserted by test_app_playing) and add `count` fresh.
    app.world_mut().resource_mut::<ActiveHazards>().clear();
    for _ in 0..count {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Resonance);
    }
}

fn wave_query_snapshot(app: &mut App) -> Vec<(ResonanceWave, Vec2)> {
    let mut q = app.world_mut().query::<(&ResonanceWave, &Position2D)>();
    q.iter(app.world()).map(|(w, p)| (*w, p.0)).collect()
}

// ── D1 — old kills pruned before threshold check ────────────────────────

#[test]
fn d1_old_kills_are_pruned_before_threshold_check() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    // Pre-seed two stale kills at t=0.0 and t=0.1.
    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::ZERO), (0.1, Vec2::ZERO)];

    // Advance clock to ~0.7s.
    tick_n(&mut app, 7, 0.1);

    // Write a new kill at time ~0.8 after the next 0.1s tick.
    write_cell_destroyed(&mut app, Vec2::new(5.0, 5.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        1,
        "Stale entries (t=0.0, t=0.1) should be pruned; only the new kill survives; got {:?}",
        tracker.kills
    );

    let waves = wave_query_snapshot(&mut app);
    assert!(
        waves.is_empty(),
        "No wave should spawn when tracker.len ≤ threshold, got {waves:?}"
    );
}

// ── D2 — pruning uses effective_window(stacks); wave spawns at stack 3 ─

#[test]
fn d2_pruning_uses_effective_window_and_spawns_wave_at_stack_three() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 3);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.1, Vec2::ZERO), (0.3, Vec2::ZERO)];

    // Advance to ~1.0s.
    tick_n(&mut app, 10, 0.1);

    write_cell_destroyed(&mut app, Vec2::new(9.0, 9.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(
        waves.len(),
        1,
        "One wave expected (3 kills in 1.1s window), got {waves:?}"
    );
    let (_, pos) = waves[0];
    assert!(
        (pos - Vec2::new(9.0, 9.0)).length() < f32::EPSILON,
        "Wave should spawn at newest kill position (9.0, 9.0), got {pos:?}"
    );
    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        2,
        "Post-spawn trim should leave exactly 2 carryover entries, got {:?}",
        tracker.kills
    );
}

// ── D3 — 2 kills within window → no wave (strictly-greater) ─────────────

#[test]
fn d3_two_kills_at_threshold_do_not_spawn_wave() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills = vec![(0.0, Vec2::ZERO)];

    write_cell_destroyed(&mut app, Vec2::new(50.0, 50.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(tracker.kills.len(), 2, "Tracker should retain both kills");

    let waves = wave_query_snapshot(&mut app);
    assert!(
        waves.is_empty(),
        "2 kills within window → no wave (threshold is strict > check); got {waves:?}"
    );
}

// ── D4 — 3rd kill → one wave, tracker trim ──────────────────────────────

#[test]
fn d4_third_kill_spawns_one_wave_at_latest_position_and_trims_tracker() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    let _breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(1.0, 1.0)), (0.1, Vec2::new(2.0, 2.0))];

    write_cell_destroyed(&mut app, Vec2::new(75.0, 300.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(
        waves.len(),
        1,
        "Exactly one wave should spawn; got {} entities",
        waves.len()
    );
    let (wave, pos) = waves[0];
    assert!(
        (pos - Vec2::new(75.0, 300.0)).length() < f32::EPSILON,
        "Wave Position2D must equal triggering kill's victim_pos, got {pos:?}"
    );
    let cfg = canonical_config();
    assert!((wave.speed - cfg.wave_speed).abs() < f32::EPSILON);
    assert!((wave.slow_duration - cfg.base_slow_duration).abs() < 1e-5);
    assert!((wave.slow_strength - cfg.base_slow_strength).abs() < 1e-5);
    assert!((wave.max_lifetime - cfg.wave_max_lifetime).abs() < f32::EPSILON);
    assert!((wave.contact_threshold - cfg.contact_threshold).abs() < f32::EPSILON);
    assert!(
        (wave.age - 0.0).abs() < f32::EPSILON,
        "Wave age should start at 0.0, got {}",
        wave.age
    );
    assert!(
        (wave.target_pos - Vec2::new(0.0, 50.0)).length() < f32::EPSILON,
        "target_pos should equal breaker's Position2D at spawn, got {:?}",
        wave.target_pos
    );

    // Confirm the wave entity has `CleanupOnExit<NodeState>`.
    let mut q = app
        .world_mut()
        .query_filtered::<Entity, (With<ResonanceWave>, With<CleanupOnExit<NodeState>>)>();
    assert_eq!(
        q.iter(app.world()).count(),
        1,
        "Wave entity must carry CleanupOnExit<NodeState>"
    );

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        2,
        "Post-spawn trim should retain 2 most-recent entries (kills_to_trigger), got {:?}",
        tracker.kills
    );
}

// ── D5 — 4th kill in one frame with 3 primed → 2 waves ─────────────────

#[test]
fn d5_two_waves_spawn_when_tracker_has_four_after_insert() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills = vec![
        (0.0, Vec2::new(1.0, 1.0)),
        (0.05, Vec2::new(2.0, 2.0)),
        (0.1, Vec2::new(3.0, 3.0)),
    ];

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(
        waves.len(),
        2,
        "Expected 2 waves (excess = 4 - 2 = 2), got {}",
        waves.len()
    );
    let positions: Vec<Vec2> = waves.iter().map(|(_, p)| *p).collect();
    for expected in [Vec2::new(3.0, 3.0), Vec2::new(10.0, 10.0)] {
        assert!(
            positions
                .iter()
                .any(|p| (*p - expected).length() < f32::EPSILON),
            "Expected wave at {expected:?} in {positions:?}"
        );
    }
    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        2,
        "Post-spawn trim retains 2 most-recent entries, got {:?}",
        tracker.kills
    );
}

// ── D6 — 2 same-frame kills at threshold → 2 waves ─────────────────────

#[test]
fn d6_two_same_frame_kills_from_threshold_spawn_two_waves() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(1.0, 1.0)), (0.1, Vec2::new(2.0, 2.0))];

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    write_cell_destroyed(&mut app, Vec2::new(20.0, 20.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(
        waves.len(),
        2,
        "Expected 2 waves (4 total kills, excess=2), got {}",
        waves.len()
    );
    let positions: Vec<Vec2> = waves.iter().map(|(_, p)| *p).collect();
    for expected in [Vec2::new(10.0, 10.0), Vec2::new(20.0, 20.0)] {
        assert!(
            positions
                .iter()
                .any(|p| (*p - expected).length() < f32::EPSILON),
            "Expected wave at {expected:?} in {positions:?}"
        );
    }
    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        2,
        "Post-spawn trim keeps 2 most-recent, got {:?}",
        tracker.kills
    );
}

// ── D7 — kill outside window does not trigger wave ──────────────────────

#[test]
fn d7_kill_outside_window_does_not_spawn_wave() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::ZERO), (0.1, Vec2::ZERO)];

    tick_n(&mut app, 7, 0.1);
    write_cell_destroyed(&mut app, Vec2::new(9.0, 9.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        1,
        "Old entries should be pruned; only new kill remains, got {:?}",
        tracker.kills
    );
    let waves = wave_query_snapshot(&mut app);
    assert!(waves.is_empty(), "Single kill → no wave, got {waves:?}");
}

// ── D8 — window widens with stacks — 1.0s-old kill counts at stack 3 ──

#[test]
fn d8_window_widens_with_stacks_old_kill_counts_at_stack_three() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 3);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.1, Vec2::new(4.0, 4.0)), (0.2, Vec2::new(5.0, 5.0))];

    tick_n(&mut app, 10, 0.1);
    write_cell_destroyed(&mut app, Vec2::new(0.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(
        waves.len(),
        1,
        "One wave expected at stack 3 with 3 kills in 1.1s window, got {}",
        waves.len()
    );
    let (_, pos) = waves[0];
    assert!(
        (pos - Vec2::new(0.0, 0.0)).length() < f32::EPSILON,
        "Wave should spawn at most-recent kill position (0.0, 0.0), got {pos:?}"
    );
    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        2,
        "Post-spawn trim to 2 entries, got {:?}",
        tracker.kills
    );
}

// ── D9 — slow values reflect current stack count ────────────────────────

#[test]
fn d9_wave_slow_fields_use_current_stack_count() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 3);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::ZERO), (0.1, Vec2::ZERO)];

    write_cell_destroyed(&mut app, Vec2::new(0.0, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(waves.len(), 1, "One wave expected, got {}", waves.len());
    let (wave, _) = waves[0];

    let cfg = canonical_config();
    let exp_dur = cfg.effective_slow_duration(3);
    let exp_str = cfg.effective_slow_strength(3);
    assert!(
        (wave.slow_duration - exp_dur).abs() < 1e-4,
        "slow_duration at stack 3 should be ~{exp_dur}, got {}",
        wave.slow_duration
    );
    assert!(
        (wave.slow_strength - exp_str).abs() < 1e-4,
        "slow_strength at stack 3 should be ~{exp_str}, got {}",
        wave.slow_strength
    );
    assert!(
        (wave.speed - cfg.wave_speed).abs() < f32::EPSILON,
        "Speed does not scale with stacks, got {}",
        wave.speed
    );
}

// ── D10 — wave target_pos = breaker at spawn ───────────────────────────

#[test]
fn d10_wave_target_pos_snapshots_breaker_position_at_spawn() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(-100.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(5.0, 5.0)), (0.1, Vec2::new(6.0, 6.0))];

    write_cell_destroyed(&mut app, Vec2::new(0.0, 300.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(waves.len(), 1);
    let (wave, _) = waves[0];
    assert!(
        (wave.target_pos - Vec2::new(-100.0, 50.0)).length() < f32::EPSILON,
        "target_pos should snapshot breaker position (-100, 50), got {:?}",
        wave.target_pos
    );
}

// ── D11 — no breaker present → no wave spawned, tracker grows ──────────

#[test]
fn d11_no_breaker_present_skips_spawn_but_records_kill() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    // NO breaker spawned.

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(1.0, 1.0)), (0.1, Vec2::new(2.0, 2.0))];

    write_cell_destroyed(&mut app, Vec2::new(0.0, 300.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        3,
        "Tracker still records the kill; got {:?}",
        tracker.kills
    );
    let waves = wave_query_snapshot(&mut app);
    assert!(waves.is_empty(), "No breaker → no wave, got {waves:?}");
}

// ── D12 — multiple breakers → wave target_pos matches one of them ──────

#[test]
fn d12_multiple_breakers_target_pos_matches_one_of_them() {
    let mut app = test_app_playing();
    app.add_systems(
        FixedUpdate,
        (resonance_track_kills, resonance_spawn_waves).chain(),
    );
    add_stack(&mut app, 1);
    spawn_breaker_at(&mut app, Vec2::new(-50.0, 50.0));
    spawn_breaker_at(&mut app, Vec2::new(50.0, 50.0));

    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(1.0, 1.0)), (0.1, Vec2::new(2.0, 2.0))];

    write_cell_destroyed(&mut app, Vec2::new(0.0, 300.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let waves = wave_query_snapshot(&mut app);
    assert_eq!(
        waves.len(),
        1,
        "Exactly one wave regardless of breaker count"
    );
    let (wave, _) = waves[0];
    let matches_left = (wave.target_pos - Vec2::new(-50.0, 50.0)).length() < f32::EPSILON;
    let matches_right = (wave.target_pos - Vec2::new(50.0, 50.0)).length() < f32::EPSILON;
    assert!(
        matches_left || matches_right,
        "target_pos should match one of the two breakers, got {:?}",
        wave.target_pos
    );
}
