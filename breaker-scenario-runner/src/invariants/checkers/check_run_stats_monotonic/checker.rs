use bevy::prelude::*;
use breaker::run::RunStats;

use crate::{invariants::*, types::InvariantKind};

/// Snapshot of [`RunStats`] numeric counters from the previous frame.
#[derive(Default, Clone, Copy)]
pub struct PreviousRunStats {
    nodes_cleared: u32,
    cells_destroyed: u32,
    bumps_performed: u32,
    perfect_bumps: u32,
    bolts_lost: u32,
}

impl PreviousRunStats {
    fn from_run_stats(stats: &RunStats) -> Self {
        Self {
            nodes_cleared: stats.nodes_cleared,
            cells_destroyed: stats.cells_destroyed,
            bumps_performed: stats.bumps_performed,
            perfect_bumps: stats.perfect_bumps,
            bolts_lost: stats.bolts_lost,
        }
    }

    /// Returns `true` if all counters are zero (default / fresh run state).
    fn is_default(&self) -> bool {
        self.nodes_cleared == 0
            && self.cells_destroyed == 0
            && self.bumps_performed == 0
            && self.perfect_bumps == 0
            && self.bolts_lost == 0
    }
}

/// Checks that [`RunStats`] counters never decrease frame-to-frame.
///
/// Stores a [`PreviousRunStats`] snapshot in a `Local`. On each tick:
/// - If [`RunStats`] is absent, resets the snapshot and skips.
/// - If the snapshot is at default (new run), seeds the snapshot and skips.
/// - Otherwise, compares each counter to the previous value and records a
///   [`ViolationEntry`] for any counter that decreased.
///
/// Detects bugs where stats are accidentally reset mid-node or decremented
/// when they should only ever increase.
pub fn check_run_stats_monotonic(
    stats: Option<Res<RunStats>>,
    mut previous: Local<Option<PreviousRunStats>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let Some(stats) = stats else {
        *previous = None;
        return;
    };

    let current = PreviousRunStats::from_run_stats(&stats);

    let Some(prev) = *previous else {
        // First tick with RunStats present — seed the snapshot, no check.
        *previous = Some(current);
        return;
    };

    // When stats return to default after being non-default, this indicates a new
    // run started. Reset snapshot to seed fresh (no violation on run transition).
    if current.is_default() && !prev.is_default() {
        *previous = Some(current);
        return;
    }

    // Check each monotonic counter.
    if current.nodes_cleared < prev.nodes_cleared {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::RunStatsMonotonic,
            entity: None,
            message: format!(
                "RunStatsMonotonic FAIL frame={} nodes_cleared decreased {} → {}",
                frame.0, prev.nodes_cleared, current.nodes_cleared,
            ),
        });
    }
    if current.cells_destroyed < prev.cells_destroyed {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::RunStatsMonotonic,
            entity: None,
            message: format!(
                "RunStatsMonotonic FAIL frame={} cells_destroyed decreased {} → {}",
                frame.0, prev.cells_destroyed, current.cells_destroyed,
            ),
        });
    }
    if current.bumps_performed < prev.bumps_performed {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::RunStatsMonotonic,
            entity: None,
            message: format!(
                "RunStatsMonotonic FAIL frame={} bumps_performed decreased {} → {}",
                frame.0, prev.bumps_performed, current.bumps_performed,
            ),
        });
    }
    if current.perfect_bumps < prev.perfect_bumps {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::RunStatsMonotonic,
            entity: None,
            message: format!(
                "RunStatsMonotonic FAIL frame={} perfect_bumps decreased {} → {}",
                frame.0, prev.perfect_bumps, current.perfect_bumps,
            ),
        });
    }
    if current.bolts_lost < prev.bolts_lost {
        log.0.push(ViolationEntry {
            frame: frame.0,
            invariant: InvariantKind::RunStatsMonotonic,
            entity: None,
            message: format!(
                "RunStatsMonotonic FAIL frame={} bolts_lost decreased {} → {}",
                frame.0, prev.bolts_lost, current.bolts_lost,
            ),
        });
    }

    *previous = Some(current);
}
