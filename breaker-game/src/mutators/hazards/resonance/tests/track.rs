//! Group C — `resonance_track_kills` timestamp + position recording.

use std::time::Duration;

use bevy::prelude::*;

use super::{
    super::system::{ResonanceTracker, ResonanceWave, resonance_track_kills},
    helpers::{test_app_playing, tick_n, tick_with_dt, write_cell_destroyed},
};

// ── C1 — single Destroyed<Cell> records one (timestamp, pos) tuple ─────

#[test]
fn c1_single_destroyed_records_one_entry() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_track_kills);

    // Advance clock so elapsed_secs ~0.5.
    tick_n(&mut app, 5, 0.1);

    write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        1,
        "Exactly one kill should be recorded, got {}",
        tracker.kills.len()
    );
    let (t, pos) = tracker.kills[0];
    let observed = app.world().resource::<Time<Fixed>>().elapsed_secs();
    assert!(
        (t - observed).abs() < 1e-3,
        "timestamp should be close to observed elapsed_secs {observed}, got {t}"
    );
    assert!(
        (pos - Vec2::new(100.0, 200.0)).length() < f32::EPSILON,
        "position should equal the written victim_pos, got {pos:?}"
    );
}

// ── C2 — no messages → no kills, no waves ──────────────────────────────

#[test]
fn c2_no_destroyed_messages_leaves_tracker_empty_and_no_waves() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_track_kills);

    tick_n(&mut app, 3, 0.1);

    let tracker = app.world().resource::<ResonanceTracker>();
    assert!(
        tracker.kills.is_empty(),
        "No messages → tracker.kills should stay empty, found {:?}",
        tracker.kills
    );

    let mut query = app.world_mut().query::<&ResonanceWave>();
    assert_eq!(
        query.iter(app.world()).count(),
        0,
        "No messages → no ResonanceWave entities"
    );
}

// ── C3 — multiple Destroyed in one frame → same timestamp, distinct pos ─

#[test]
fn c3_multiple_destroyed_same_frame_share_timestamp_distinct_positions() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, resonance_track_kills);

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    write_cell_destroyed(&mut app, Vec2::new(20.0, 20.0));
    write_cell_destroyed(&mut app, Vec2::new(30.0, 30.0));

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        3,
        "3 messages in one frame should produce 3 entries, got {}",
        tracker.kills.len()
    );

    let t0 = tracker.kills[0].0;
    for (t, _) in &tracker.kills {
        assert!(
            (t - t0).abs() < f32::EPSILON,
            "All same-frame timestamps should match {t0}, got {t}"
        );
    }

    let positions: Vec<Vec2> = tracker.kills.iter().map(|(_, p)| *p).collect();
    for expected in [
        Vec2::new(10.0, 10.0),
        Vec2::new(20.0, 20.0),
        Vec2::new(30.0, 30.0),
    ] {
        assert!(
            positions
                .iter()
                .any(|p| (*p - expected).length() < f32::EPSILON),
            "Expected position {expected:?} in {positions:?}"
        );
    }
}
