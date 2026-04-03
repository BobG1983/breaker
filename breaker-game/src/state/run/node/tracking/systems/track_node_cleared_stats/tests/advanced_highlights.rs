//! Tests for advanced highlight types: `SpeedDemon`, `Untouchable`,
//! `Comeback`, `PerfectNode`, and regression tests.

use bevy::prelude::*;

use super::helpers::*;
use crate::state::run::{
    definition::HighlightConfig,
    resources::{HighlightKind, HighlightTracker, RunStats},
};

// --- Behavior 9: SpeedDemon detected when elapsed < config.speed_demon_secs ---

#[test]
fn speed_demon_detected_when_elapsed_below_threshold() {
    let mut app = test_app();
    let config = HighlightConfig {
        speed_demon_secs: 5.0,
        ..Default::default()
    };
    app.insert_resource(config);

    // Advance a few ticks to build up elapsed time, then set node_start_time
    // relative to the actual elapsed so the delta is exactly 4.5s.
    app.insert_resource(TestNodeCleared(false));
    for _ in 0..10 {
        tick(&mut app);
    }

    let current_time = app.world().resource::<Time<Fixed>>().elapsed_secs();
    app.world_mut()
        .resource_mut::<HighlightTracker>()
        .node_start_time = current_time - 4.5;

    // Now send the NodeCleared
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let speed_demon = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::SpeedDemon);
    assert!(
        speed_demon.is_some(),
        "should detect SpeedDemon when node elapsed (~4.5s) < config.speed_demon_secs (5.0)"
    );
}

// --- Behavior 10: SpeedDemon NOT detected when elapsed >= threshold ---

#[test]
fn speed_demon_not_detected_when_elapsed_above_threshold() {
    let mut app = test_app();
    let config = HighlightConfig {
        speed_demon_secs: 5.0,
        ..Default::default()
    };
    app.insert_resource(config);

    // Advance a few ticks, then set node_start_time so elapsed is exactly 6.0s.
    app.insert_resource(TestNodeCleared(false));
    for _ in 0..10 {
        tick(&mut app);
    }

    let current_time = app.world().resource::<Time<Fixed>>().elapsed_secs();
    app.world_mut()
        .resource_mut::<HighlightTracker>()
        .node_start_time = current_time - 6.0;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let speed_demon = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::SpeedDemon);
    assert!(
        speed_demon.is_none(),
        "should NOT detect SpeedDemon when node elapsed (~6.0s) >= config.speed_demon_secs (5.0)"
    );
}

// --- Behavior 11: Untouchable detected after consecutive no-damage nodes ---

#[test]
fn untouchable_detected_after_consecutive_no_damage_nodes() {
    let mut app = test_app();
    let config = HighlightConfig {
        untouchable_nodes: 2,
        ..Default::default()
    };
    app.insert_resource(config);
    // 1 existing consecutive no-damage node + current node with 0 bolts lost = 2 >= 2
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.consecutive_no_damage_nodes = 1;
    tracker.node_bolts_lost = 0;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let untouchable = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::Untouchable);
    assert!(
        untouchable.is_some(),
        "should detect Untouchable with 1 existing + current no-damage = 2 >= config.untouchable_nodes=2"
    );
}

// --- Behavior 12: Untouchable streak resets when bolt lost ---

#[test]
fn untouchable_streak_resets_when_bolt_lost() {
    let mut app = test_app();
    let config = HighlightConfig {
        untouchable_nodes: 2,
        ..Default::default()
    };
    app.insert_resource(config);
    // Had 3 consecutive no-damage nodes, but lost a bolt this node
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.consecutive_no_damage_nodes = 3;
    tracker.node_bolts_lost = 1;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let untouchable = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::Untouchable);
    assert!(
        untouchable.is_none(),
        "should NOT detect Untouchable when node_bolts_lost=1 (streak reset)"
    );
    // Also verify the streak was reset
    let tracker = app.world().resource::<HighlightTracker>();
    assert_eq!(
        tracker.consecutive_no_damage_nodes, 0,
        "consecutive_no_damage_nodes should reset to 0 when bolt was lost"
    );
}

// --- Behavior 13: Comeback when bolts lost >= threshold ---

#[test]
fn comeback_detected_when_bolts_lost_exceeds_threshold() {
    let mut app = test_app();
    let config = HighlightConfig {
        comeback_bolts_lost: 3,
        ..Default::default()
    };
    app.insert_resource(config);
    app.world_mut()
        .resource_mut::<HighlightTracker>()
        .node_bolts_lost = 3;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let comeback = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::Comeback);
    assert!(
        comeback.is_some(),
        "should detect Comeback with node_bolts_lost=3 >= config.comeback_bolts_lost=3"
    );
}

// --- Behavior 14: PerfectNode when all bumps perfect ---

#[test]
fn perfect_node_detected_when_all_bumps_perfect() {
    let mut app = test_app();
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.non_perfect_bumps_this_node = 0;
    tracker.total_bumps_this_node = 5;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let perfect_node = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::PerfectNode);
    assert!(
        perfect_node.is_some(),
        "should detect PerfectNode when non_perfect=0 and total=5"
    );
}

// --- Behavior 15: PerfectNode NOT detected with non-perfect bumps ---

#[test]
fn perfect_node_not_detected_with_non_perfect_bumps() {
    let mut app = test_app();
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.non_perfect_bumps_this_node = 1;
    tracker.total_bumps_this_node = 5;

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let perfect_node = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::PerfectNode);
    assert!(
        perfect_node.is_none(),
        "should NOT detect PerfectNode when non_perfect_bumps_this_node=1"
    );
}

// ---------------------------------------------------------------
// Regression: stale best_perfect_streak duplicates on later nodes
// ---------------------------------------------------------------

#[test]
fn perfect_streak_not_duplicated_from_stale_cross_node_best() {
    // Bug: best_perfect_streak is a cross-node field. After node 2
    // achieves a streak of 7 (above threshold 5), every subsequent
    // NodeCleared records another PerfectStreak with value=7, even
    // if the current node had 0 consecutive perfect bumps.
    //
    // Correct behavior: PerfectStreak should only fire when the
    // current node actually contributes to the streak, not when
    // re-reading a stale cross-node value.
    let mut app = test_app();

    // Simulate state after a previous node set best_perfect_streak=7,
    // but the current node has 0 consecutive perfect bumps.
    let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
    tracker.best_perfect_streak = 7;
    tracker.consecutive_perfect_bumps = 0;

    // Default perfect_streak_count = 5, so 7 >= 5 would fire
    // if the system doesn't guard against stale values.

    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let streak = stats
        .highlights
        .iter()
        .find(|h| h.kind == HighlightKind::PerfectStreak);
    assert!(
        streak.is_none(),
        "PerfectStreak should NOT fire when the current node had 0 consecutive \
         perfect bumps, even though best_perfect_streak=7 from a prior node"
    );
}
