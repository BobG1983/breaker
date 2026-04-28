use bevy::prelude::*;
use breaker::state::run::node::{messages::IncreaseNodeTimer, resources::NodeTimer};

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// When `IncreaseNodeTimer` is sent on the same tick that the timer increases,
/// no violation should fire — the increase is legitimate (delta added back).
#[test]
fn timer_monotonically_decreasing_no_violation_when_increase_node_timer_sent() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 20.0,
        total:     60.0,
    });

    // Tick 1: seeds Local with (20.0, 60.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Timer increases to 25.0 — normally a violation.
    app.world_mut().resource_mut::<NodeTimer>().remaining = 25.0;

    // Send an IncreaseNodeTimer message — exempts this increase.
    app.world_mut()
        .resource_mut::<Messages<IncreaseNodeTimer>>()
        .write(IncreaseNodeTimer { delta: 5.0 });

    // Tick 2: remaining 20.0 → 25.0 with IncreaseNodeTimer → no violation.
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing),
        "expected no TimerMonotonicallyDecreasing violation when IncreaseNodeTimer \
        was sent on the same tick the timer increased, got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// Multiple `IncreaseNodeTimer` messages in one tick must all be drained, so
/// the next tick's checker run does not see a leaked retained message.
///
/// Regression guard: a partial-drain (`.next().is_some()`) leaves N-1 messages
/// in the reader; Bevy 0.18's double-buffer keeps them readable for one extra
/// frame, suppressing legitimate violations on the following tick.
#[test]
fn timer_monotonically_decreasing_drains_all_messages_no_leak_to_next_tick() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 20.0,
        total:     60.0,
    });

    // Tick 1: seeds Local with (20.0, 60.0)
    tick(&mut app);

    // Two IncreaseNodeTimer messages in one tick (e.g. multi-cell Siphon kill).
    {
        let mut messages = app
            .world_mut()
            .resource_mut::<Messages<IncreaseNodeTimer>>();
        messages.write(IncreaseNodeTimer { delta: 3.0 });
        messages.write(IncreaseNodeTimer { delta: 2.0 });
    }
    app.world_mut().resource_mut::<NodeTimer>().remaining = 25.0;

    // Tick 2: remaining 20.0 → 25.0 with messages present → no violation.
    tick(&mut app);

    // Tick 3: spurious increase with NO new IncreaseNodeTimer messages.
    // If the reader leaked retained messages from tick 2, the violation is
    // silently suppressed — the bug we are guarding against.
    app.world_mut().resource_mut::<NodeTimer>().remaining = 30.0;
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    let violations: Vec<&str> = log
        .0
        .iter()
        .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
        .map(|e| e.message.as_str())
        .collect();
    assert_eq!(
        violations.len(),
        1,
        "expected exactly 1 TimerMonotonicallyDecreasing violation on tick 3 \
         (where remaining increases with no IncreaseNodeTimer), got: {violations:?}",
    );
}

/// Without `IncreaseNodeTimer`, the same timer increase IS a violation.
///
/// Regression guard: verifies the exemption is not silently applied when no
/// message was sent (guards against off-by-one or always-exempt bugs).
#[test]
fn timer_monotonically_decreasing_fires_when_no_increase_node_timer_despite_increase() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 20.0,
        total:     60.0,
    });

    // Tick 1: seeds Local with (20.0, 60.0)
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Timer increases — no IncreaseNodeTimer → violation.
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
        increases without IncreaseNodeTimer, got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::TimerMonotonicallyDecreasing)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}
