use breaker::state::run::node::resources::NodeTimer;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// Timer increases from 50.0 to 55.0 — a violation must be recorded.
///
/// Tick 1: insert `NodeTimer { remaining: 50.0, total: 60.0 }` -> seeds `Local(50.0)`.
/// Tick 2: update to `remaining: 55.0` -> fires violation.
#[test]
fn timer_monotonically_decreasing_fires_when_timer_increases() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 50.0,
        total:     60.0,
    });

    // Tick 1: seeds Local with 50.0
    tick(&mut app);

    assert!(
        app.world().resource::<ViolationLog>().0.is_empty(),
        "no violation expected after seeding tick"
    );

    // Update timer to a higher value (illegal increase)
    app.world_mut().resource_mut::<NodeTimer>().remaining = 55.0;

    // Tick 2: 55.0 > 50.0 → violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly one TimerMonotonicallyDecreasing violation, got {}",
        log.0.len()
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::TimerMonotonicallyDecreasing
    );
}

/// Timer decreasing from 50.0 to 49.0 is correct. No violation should fire.
#[test]
fn timer_monotonically_decreasing_does_not_fire_when_timer_decreases() {
    let mut app = test_app_timer_monotonic();

    app.insert_resource(NodeTimer {
        remaining: 50.0,
        total:     60.0,
    });

    // Tick 1: seeds Local with 50.0
    tick(&mut app);

    // Decrease timer (correct behavior)
    app.world_mut().resource_mut::<NodeTimer>().remaining = 49.0;

    // Tick 2: 49.0 < 50.0 → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when timer decreases from 50.0 to 49.0"
    );
}

/// When [`NodeTimer`] is not present, the system must do nothing.
#[test]
fn timer_monotonically_decreasing_skips_when_no_node_timer() {
    let mut app = test_app_timer_monotonic();
    // No NodeTimer inserted

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when NodeTimer is absent"
    );
}

/// When [`NodeTimer`] disappears and then reappears at 60.0, Local must have
/// been reset so no spurious violation fires.
#[test]
fn timer_monotonically_decreasing_resets_local_when_timer_removed() {
    let mut app = test_app_timer_monotonic();

    // Start with NodeTimer present
    app.insert_resource(NodeTimer {
        remaining: 50.0,
        total:     60.0,
    });

    // Tick 1: seeds Local with 50.0
    tick(&mut app);

    // Remove NodeTimer → system should reset Local
    app.world_mut().remove_resource::<NodeTimer>();

    // Tick 2: NodeTimer absent → no violation, Local reset
    tick(&mut app);

    let log_after_removal = app.world().resource::<ViolationLog>();
    assert!(
        log_after_removal.0.is_empty(),
        "expected no violation when NodeTimer is absent"
    );

    // Reinsert NodeTimer at 60.0 (higher than old 50.0, but Local was reset)
    app.insert_resource(NodeTimer {
        remaining: 60.0,
        total:     60.0,
    });

    // Tick 3: 60.0 appears fresh — no previous value → no violation
    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when NodeTimer reappears after reset (Local was cleared)"
    );
}
