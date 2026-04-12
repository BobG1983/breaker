//! Tests for config-driven threshold overrides: `clutch_clear_secs`,
//! `fast_clear_fraction`, and `perfect_streak_count`.

use super::helpers::*;
use crate::state::run::{
    definition::HighlightConfig,
    node::resources::NodeTimer,
    resources::{HighlightKind, HighlightTracker, RunStats},
};

// --- Behavior 4: clutch_clear_secs reads from config ---

#[test]
fn clutch_clear_uses_config_threshold() {
    let mut app = test_app();
    // Override config with clutch_clear_secs=5.0
    let config = HighlightConfig {
        clutch_clear_secs: 5.0,
        ..Default::default()
    };
    app.insert_resource(config);
    // remaining=4.5 < 5.0 -> ClutchClear
    app.insert_resource(NodeTimer {
        remaining: 4.5,
        total:     30.0,
    });
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let clutch = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::ClutchClear);
    assert!(
        clutch.is_some(),
        "should detect ClutchClear with config.clutch_clear_secs=5.0 and remaining=4.5"
    );
    assert!(
        (clutch.unwrap().value - 4.5).abs() < f32::EPSILON,
        "ClutchClear value should be 4.5"
    );
}

// --- Behavior 5: fast_clear_fraction reads from config ---

#[test]
fn fast_clear_uses_config_fraction() {
    let mut app = test_app();
    // Override config with fast_clear_fraction=0.3
    let config = HighlightConfig {
        fast_clear_fraction: 0.3,
        ..Default::default()
    };
    app.insert_resource(config);
    // total=30, remaining=22 -> elapsed=8, threshold=30*0.3=9.0, 8<9 -> FastClear
    app.insert_resource(NodeTimer {
        remaining: 22.0,
        total:     30.0,
    });
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let fast = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::FastClear);
    assert!(
        fast.is_some(),
        "should detect FastClear with config.fast_clear_fraction=0.3, elapsed=8 < 9"
    );
}

// --- Behavior 6: perfect_streak_count reads from config ---

#[test]
fn perfect_streak_uses_config_count() {
    let mut app = test_app();
    // Override config with perfect_streak_count=3
    let config = HighlightConfig {
        perfect_streak_count: 3,
        ..Default::default()
    };
    app.insert_resource(config);
    // Set streak to 4 which exceeds config threshold of 3
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.best_perfect_streak = 4;
    tracker.consecutive_perfect_bumps = 4;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let streak = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::PerfectStreak);
    assert!(
        streak.is_some(),
        "should detect PerfectStreak with config.perfect_streak_count=3 and streak=4"
    );
    assert!(
        (streak.unwrap().value - 4.0).abs() < f32::EPSILON,
        "PerfectStreak value should be 4.0"
    );
}
