//! Tests for basic stat detection: `nodes_cleared`, `ClutchClear`,
//! `NoDamageNode`, `FastClear`, and `PerfectStreak`.

use super::helpers::*;
use crate::state::run::{
    node::resources::NodeTimer,
    resources::{HighlightKind, HighlightTracker, NodeOutcome, RunStats},
};

#[test]
fn increments_nodes_cleared() {
    let mut app = test_app();
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    assert_eq!(
        stats.nodes_cleared, 1,
        "NodeCleared should increment nodes_cleared"
    );
}

#[test]
fn detects_clutch_clear_when_timer_below_threshold() {
    let mut app = test_app();
    app.insert_resource(NodeTimer {
        remaining: 2.5,
        total: 30.0,
    });
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 3;
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let clutch = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::ClutchClear);
    assert!(
        clutch.is_some(),
        "should detect ClutchClear with 2.5s remaining"
    );
    let clutch = clutch.unwrap();
    assert_eq!(clutch.node_index, 3);
    assert!(
        (clutch.value - 2.5).abs() < f32::EPSILON,
        "ClutchClear value should be 2.5, got {}",
        clutch.value
    );
}

#[test]
fn no_clutch_clear_when_timer_above_threshold() {
    let mut app = test_app();
    // Default clutch_clear_secs is 3.0, so 5.0 remaining should not trigger
    app.insert_resource(NodeTimer {
        remaining: 5.0,
        total: 30.0,
    });
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let clutch = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::ClutchClear);
    assert!(
        clutch.is_none(),
        "should NOT detect ClutchClear with 5.0s remaining (>= 3.0)"
    );
}

#[test]
fn detects_no_damage_node_when_no_bolts_lost() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<HighlightTracker>()
        .node_bolts_lost = 0;
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 2;
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let no_damage = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::NoDamageNode);
    assert!(
        no_damage.is_some(),
        "should detect NoDamageNode when node_bolts_lost == 0"
    );
    assert_eq!(no_damage.unwrap().node_index, 2);
}

#[test]
fn no_damage_node_not_detected_when_bolts_lost() {
    let mut app = test_app();
    app.world_mut()
        .resource_mut::<HighlightTracker>()
        .node_bolts_lost = 1;
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let no_damage = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::NoDamageNode);
    assert!(
        no_damage.is_none(),
        "should NOT detect NoDamageNode when bolts were lost"
    );
}

#[test]
fn detects_fast_clear_when_elapsed_below_half_total() {
    let mut app = test_app();
    // remaining=20 means elapsed=10, total=30 -> 10/30 = 0.33 < 0.5 (default fast_clear_fraction)
    app.insert_resource(NodeTimer {
        remaining: 20.0,
        total: 30.0,
    });
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let fast = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::FastClear);
    assert!(
        fast.is_some(),
        "should detect FastClear when elapsed (10) < 0.5 of total (30)"
    );
    assert_eq!(fast.unwrap().node_index, 1);
}

#[test]
fn detects_perfect_streak_at_node_end() {
    let mut app = test_app();
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.best_perfect_streak = 7;
    tracker.consecutive_perfect_bumps = 7;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let streak = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::PerfectStreak);
    assert!(
        streak.is_some(),
        "should detect PerfectStreak with streak of 7 (>= 5)"
    );
    assert!(
        (streak.unwrap().value - 7.0).abs() < f32::EPSILON,
        "PerfectStreak value should be 7.0"
    );
}
