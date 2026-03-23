# Phase 4i: Run Stats & Summary

**Goal**: Track run statistics and display a full summary on the run-end screen.

**Wave**: 4 (capstone) — parallel with 4h. **Session 8.**

## Dependencies

- 4e (Node Escalation) — need multi-node runs to track
- 4f (Chip Offerings) — need chip collection to track

## What to Build

### RunStats Resource

Hybrid approach: a dedicated `RunStats` resource populated by observing existing messages.

```rust
#[derive(Resource, Default)]
pub struct RunStats {
    pub nodes_cleared: u32,
    pub cells_destroyed: u32,
    pub bumps_performed: u32,
    pub perfect_bumps: u32,
    pub bolts_lost: u32,
    pub chips_collected: Vec<String>,   // names in order collected
    pub evolutions_performed: u32,
    pub time_elapsed: f32,
    pub seed: u64,
    pub highlights: Vec<RunHighlight>,
}

#[derive(Clone, Debug)]
pub struct RunHighlight {
    pub kind: HighlightKind,
    pub node_index: u32,
    pub value: f32,         // context-dependent (time remaining, streak count, etc.)
}

pub enum HighlightKind {
    ClutchClear,        // node cleared with < 3s remaining
    MassDestruction,    // 10+ cells destroyed within a 1-second window
    PerfectStreak,      // 5+ consecutive perfect bumps
    FastClear,          // node cleared in < 50% of allotted time
    FirstEvolution,     // first evolution performed this run
    NoDamageNode,       // node cleared without losing a bolt
}
```

### Message Observation Systems

Systems that listen to existing messages and increment `RunStats`:
- `CellDestroyed` -> `cells_destroyed += 1`
- `BumpPerformed { grade }` -> `bumps_performed += 1`, if perfect: `perfect_bumps += 1`
- `BoltLost` -> `bolts_lost += 1`
- `ChipSelected { name }` -> `chips_collected.push(name)`
- Node cleared -> `nodes_cleared += 1`
- Time tracking: accumulate `time.delta_secs()` each frame during `Playing`

No new mutation paths — purely observational.

### Run-End Screen Enhancement

Current run-end screen shows win/lose. Enhance with:

- **Outcome**: WIN / LOSE (prominent)
- **Stats grid**: Nodes cleared, cells destroyed, bumps, perfect bumps, bolts lost, time elapsed
- **Build summary**: List of chips collected (with stack counts), evolutions performed
- **Flux earned**: Calculated from stats (formula TBD — even if Flux spending isn't built yet, show the earned amount)
- **Seed**: Displayed for sharing/replay
- **Highlights**: Top 3 highlight moments from the run (e.g., "Clutch Clear — Node 7, 0.3s remaining", "Mass Destruction — 11 cells in one shockwave")
- **Actions**: Retry (same seed), New Run, Main Menu

### Flux Calculation

Simple formula for the vertical slice (will be refined):
- Base: nodes_cleared * 10
- Bonus: perfect_bumps * 2
- Bonus: evolutions_performed * 25
- Penalty: bolts_lost * -3
- Minimum: 0

Flux is *earned and displayed* but not *spendable* until Phase 8 (Roguelite Progression).

## Scenario Coverage

### New Invariants
- **`RunStatsMonotonic`** — counters in `RunStats` (nodes_cleared, cells_destroyed, bumps_performed, perfect_bumps, bolts_lost) never decrease. Implemented in runner as `InvariantKind::RunStatsMonotonic`.
- **`FluxNonNegative`** — Flux is guaranteed non-negative via `saturating_sub` in `RunStats::flux_earned()`. No separate invariant implemented — the invariant is structural.

### New Scenarios
- `mechanic/run_stats_accumulation.scenario.ron` — Scripted multi-node run that clears nodes, loses bolts, performs bumps. At run end, verify `RunStats` counters match expected values (known inputs → known outputs).
- Existing multinode stress scenarios should verify `RunStatsMonotonic` as an additional invariant.

## Acceptance Criteria

1. RunStats accumulates throughout the run via message observation
2. Run-end screen displays all tracked stats
3. Chip build summary shows what was collected and at what stacks
4. Flux earned is calculated and displayed
5. Seed is displayed on run-end screen
6. Retry option starts a new run with the same seed
7. At least 3 highlight moment types are detected and displayed on the run-end screen
