use bevy::prelude::*;
use breaker::run::node::{messages::ReverseTimePenalty, resources::NodeTimer};

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`NodeTimer::remaining`] never increases between ticks.
///
/// Stores `(remaining, total)` from the previous tick in a `Local`. Resets when
/// `total` changes (node transition) or when `remaining` jumps back near `total`
/// (same-duration node transition). If `remaining` increases otherwise, appends a
/// [`ViolationEntry`] with [`InvariantKind::TimerMonotonicallyDecreasing`].
///
/// **`ReverseTimePenalty` exemption**: when a [`ReverseTimePenalty`] message is
/// present in the current tick, a timer increase is expected (the effect adds
/// seconds back) and is silently skipped. This prevents false positives when the
/// `TimePenalty` effect is reversed (e.g. on node end or effect expiry).
///
/// Skips and resets when [`NodeTimer`] is absent.
pub fn check_timer_monotonically_decreasing(
    timer: Option<Res<NodeTimer>>,
    mut previous: Local<Option<(f32, f32)>>,
    frame: Res<ScenarioFrame>,
    mut reverse_reader: MessageReader<ReverseTimePenalty>,
    mut log: ResMut<ViolationLog>,
) {
    // Consume all ReverseTimePenalty messages this tick.
    // If any were sent, a timer increase is legitimate.
    let reverse_penalty_this_tick = reverse_reader.read().next().is_some();

    let Some(timer) = timer else {
        *previous = None;
        return;
    };
    let current = timer.remaining;
    let current_total = timer.total;
    if let Some((prev_remaining, prev_total)) = *previous {
        if (current_total - prev_total).abs() > f32::EPSILON {
            // Node transition — total changed, reset tracking
            *previous = Some((current, current_total));
            return;
        }
        if current > prev_remaining {
            // Check if this looks like a freshly initialized timer (new node
            // with the same duration). On the first tick of a new node,
            // remaining ≈ total. A real intra-node bug would have remaining
            // somewhere in the middle, not near total.
            let near_total = (current - current_total).abs() < 1.0;
            if near_total {
                // Same-duration node transition — reset tracking
                *previous = Some((current, current_total));
                return;
            }
            // Legitimate increase from ReverseTimePenalty — skip, don't fire.
            if reverse_penalty_this_tick {
                *previous = Some((current, current_total));
                return;
            }
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::TimerMonotonicallyDecreasing,
                entity: None,
                message: format!(
                    "TimerMonotonicallyDecreasing FAIL frame={} remaining increased {prev_remaining:.3} → {current:.3}",
                    frame.0,
                ),
            });
        }
    }
    *previous = Some((current, current_total));
}

#[cfg(test)]
mod tests {
    use breaker::run::node::messages::ReverseTimePenalty;

    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_timer_monotonic() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ReverseTimePenalty>()
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_timer_monotonically_decreasing);
        app
    }

    /// Timer increases from 50.0 to 55.0 — a violation must be recorded.
    ///
    /// Tick 1: insert `NodeTimer { remaining: 50.0, total: 60.0 }` → seeds `Local(50.0)`.
    /// Tick 2: update to `remaining: 55.0` → fires violation.
    #[test]
    fn timer_monotonically_decreasing_fires_when_timer_increases() {
        let mut app = test_app_timer_monotonic();

        app.insert_resource(NodeTimer {
            remaining: 50.0,
            total: 60.0,
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
            total: 60.0,
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
            total: 60.0,
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
            total: 60.0,
        });

        // Tick 3: 60.0 appears fresh — no previous value → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when NodeTimer reappears after reset (Local was cleared)"
        );
    }

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
            increases (10.0 → 15.0) with same total (30.0), got: {:?}",
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
        // (59.984 = 60.0 - one fixed timestep ≈ 1/64)
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
            transition (remaining 53.7 → 59.984, total unchanged at 60.0), got: {:?}",
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
}
