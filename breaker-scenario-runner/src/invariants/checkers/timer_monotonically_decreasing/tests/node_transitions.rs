use bevy::prelude::*;
use breaker::run::node::resources::NodeTimer;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// When `NodeTimer` changes to a new timer with a different `total`, the
/// increase in `remaining` represents a node transition, not a violation.
#[test]
fn timer_monotonically_decreasing_no_violation_when_remaining_increases_with_new_total() {
    let mut app = test_app_timer_monotonic();

    // Start: NodeTimer { remaining: 5.0, total: 30.0 }
    app.insert_resource(NodeTimer {
        remaining: 5.0,
        total: 30.0,
    });

    // Tick 1: seeds Local with (5.0, 30.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Node transition: new timer with different total
    app.world_mut().resource_mut::<NodeTimer>().remaining = 25.0;
    app.world_mut().resource_mut::<NodeTimer>().total = 45.0;

    // Tick 2: remaining went from 5.0 to 25.0, BUT total changed from 30.0 to 45.0
    // → node transition; Local resets → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing),
        "expected no TimerMonotonicallyDecreasing violation when remaining increases \
        because total also changed (node transition), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// When `NodeTimer.remaining` increases while `total` stays the same,
/// a violation fires — within the same node, the timer should only decrease.
#[test]
fn timer_monotonically_decreasing_fires_when_remaining_increases_with_same_total() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 10.0,
        total: 30.0,
    });

    // Tick 1: seeds Local with (10.0, 30.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Remaining goes up while total stays the same — illegal within same node
    app.world_mut().resource_mut::<NodeTimer>().remaining = 15.0;
    // total remains 30.0

    // Tick 2: remaining 10.0 → 15.0, total unchanged → violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .count(),
        1,
        "expected exactly 1 TimerMonotonicallyDecreasing violation when remaining \
        increases (10.0 -> 15.0) with same total (30.0), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// When two consecutive nodes have the same timer duration (same `total`),
/// the `remaining` jumps back near `total` on the first tick of the new node.
/// This is a node transition, not a violation.
#[test]
fn timer_monotonically_decreasing_no_violation_on_same_duration_node_transition() {
    let mut app = test_app_timer_monotonic();

    // First node: total=60.0, remaining starts at 53.7 (partway through)
    app.insert_resource(NodeTimer {
        remaining: 53.7,
        total: 60.0,
    });

    // Tick 1: seeds Local with (53.7, 60.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // New node with same duration: remaining resets near total
    // (59.984 = 60.0 - one fixed timestep ~ 1/64)
    app.world_mut().resource_mut::<NodeTimer>().remaining = 59.984;
    // total stays 60.0 — same duration node

    // Tick 2: remaining jumped 53.7 → 59.984, total unchanged
    // This is a node transition (remaining near total), not a violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing),
        "expected no TimerMonotonicallyDecreasing violation on same-duration node \
        transition (remaining 53.7 -> 59.984, total unchanged at 60.0), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// When `NodeTimer` starts at `remaining: 0.0` and then jumps to 80.0
/// (e.g., zero-initialized timer receives `SetTimerRemaining` mutation),
/// the invariant must detect this increase. The current guard
/// `prev_remaining > 0.0` incorrectly skips this case.
#[test]
fn timer_monotonically_decreasing_fires_when_timer_increases_from_zero() {
    let mut app = test_app_timer_monotonic();

    // Start with a zero-initialized NodeTimer (both fields zero)
    app.insert_resource(NodeTimer {
        remaining: 0.0,
        total: 0.0,
    });

    // Tick 1: seeds Local with (0.0, 0.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Remaining jumps from 0.0 to 80.0 while total stays at 0.0
    // (simulates asset-loading race: timer zero-initialized, then mutated)
    app.world_mut().resource_mut::<NodeTimer>().remaining = 80.0;

    // Tick 2: remaining 0.0 → 80.0, total unchanged at 0.0 → violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .count(),
        1,
        "expected exactly 1 TimerMonotonicallyDecreasing violation when remaining \
        increases from 0.0 to 80.0, got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}
