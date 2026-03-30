use bevy::prelude::*;
use breaker::run::node::{messages::ReverseTimePenalty, resources::NodeTimer};

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// When `ReverseTimePenalty` is sent on the same tick that the timer increases,
/// no violation should fire — the increase is legitimate (seconds added back).
#[test]
fn timer_monotonically_decreasing_no_violation_when_reverse_time_penalty_sent() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 20.0,
        total: 60.0,
    });

    // Tick 1: seeds Local with (20.0, 60.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Timer increases to 25.0 — normally a violation.
    app.world_mut().resource_mut::<NodeTimer>().remaining = 25.0;

    // Send a ReverseTimePenalty message — exempts this increase.
    app.world_mut()
        .resource_mut::<Messages<ReverseTimePenalty>>()
        .write(ReverseTimePenalty { seconds: 5.0 });

    // Tick 2: remaining 20.0 → 25.0 with ReverseTimePenalty → no violation.
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing),
        "expected no TimerMonotonicallyDecreasing violation when ReverseTimePenalty \
        was sent on the same tick the timer increased, got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// Without `ReverseTimePenalty`, the same timer increase IS a violation.
///
/// Regression guard: verifies the exemption is not silently applied when no
/// message was sent (guards against off-by-one or always-exempt bugs).
#[test]
fn timer_monotonically_decreasing_fires_when_no_reverse_penalty_despite_increase() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 20.0,
        total: 60.0,
    });

    // Tick 1: seeds Local with (20.0, 60.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Timer increases — no ReverseTimePenalty → violation.
    app.world_mut().resource_mut::<NodeTimer>().remaining = 25.0;

    // Tick 2: remaining 20.0 → 25.0, no message → violation.
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .count(),
        1,
        "expected exactly 1 TimerMonotonicallyDecreasing violation when remaining \
        increases without ReverseTimePenalty, got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}
