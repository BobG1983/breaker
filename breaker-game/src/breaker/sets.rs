//! Breaker domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the breaker domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    /// The `move_breaker` system — updates breaker position from input.
    Move,
    /// The `reset_breaker` system — resets breaker position and state on node entry.
    Reset,
    /// The `grade_bump` system — grades bump timing and writes `BumpPerformed`/`BumpWhiffed`.
    GradeBump,
    /// The `update_breaker_state` system — updates breaker state machine each tick.
    UpdateState,
    /// The `update_previous_dash_state` system — copies current `DashState`
    /// into `PreviousDashState` once per tick, after `UpdateState` has run.
    /// Systems scheduled `.after(UpdateState).before(UpdatePreviousState)` can
    /// read both the current `DashState` and the previous-tick snapshot
    /// simultaneously, enabling per-tick transition detection.
    UpdatePreviousState,
    /// The `handle_bolt_lost` system — applies the breaker's `BoltLossBehavior`
    /// when a `BoltLost` fires for it. Protocols that mutate `BoltLossBehavior`
    /// (e.g. `reckless_dash_on_dash_transition`) must be ordered
    /// `.before(BreakerSystems::HandleBoltLost)` so the mutated behavior is
    /// live when `handle_bolt_lost` reads it.
    HandleBoltLost,
    /// The `breaker_cell_collision` system — detects breaker-cell overlap.
    CellCollision,
}
