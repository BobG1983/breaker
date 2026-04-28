use bevy::prelude::*;
use breaker::state::run::node::{messages::IncreaseNodeTimer, resources::NodeTimer};

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`NodeTimer::remaining`] never increases between ticks.
///
/// Stores `(remaining, total)` from the previous tick in a `Local`. Resets when
/// `total` changes (node transition) or when `remaining` jumps back near `total`
/// (same-duration node transition). If `remaining` increases otherwise, appends a
/// [`ViolationEntry`] with [`InvariantKind::TimerMonotonicallyDecreasing`].
///
/// **`IncreaseNodeTimer` exemption**: when an [`IncreaseNodeTimer`] message is
/// present in the current tick, a timer increase is expected (the message adds
/// delta back) and is silently skipped. This prevents false positives when
/// something legitimately adds time back (Siphon protocol, future systems).
///
/// Skips and resets when [`NodeTimer`] is absent.
pub fn check_timer_monotonically_decreasing(
    timer: Option<Res<NodeTimer>>,
    mut previous: Local<Option<(f32, f32)>>,
    frame: Res<ScenarioFrame>,
    mut increase_reader: MessageReader<IncreaseNodeTimer>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    // Drain ALL IncreaseNodeTimer messages this tick. Bevy 0.18 retains
    // unread messages in the double-buffer for one extra frame, so partial
    // drains (e.g. `.next().is_some()`) can leak into next tick and cause
    // false-negative violation suppression.
    let increase_this_tick = increase_reader.read().count() > 0;

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
            // Legitimate increase from IncreaseNodeTimer — skip, don't fire.
            if increase_this_tick {
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
