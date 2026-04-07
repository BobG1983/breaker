use bevy::prelude::*;
use breaker::state::run::node::{messages::ReverseTimePenalty, resources::NodeTimer};

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
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
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
