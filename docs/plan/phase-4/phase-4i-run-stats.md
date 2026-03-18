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
struct RunStats {
    // Counters
    nodes_cleared: u32,
    cells_destroyed: u32,
    bumps_performed: u32,
    perfect_bumps: u32,
    bolts_lost: u32,
    chips_collected: Vec<String>,   // names in order collected
    chips_maxed: u32,
    evolutions_performed: u32,
    time_elapsed: f32,              // total run time
    seed: u64,

    // Highlight moments (Pillar 9: Every Run Tells a Story)
    highlights: Vec<RunHighlight>,
}

#[derive(Clone, Debug)]
struct RunHighlight {
    kind: HighlightKind,
    node_index: u32,
    value: f32,         // context-dependent (time remaining, cell count, etc.)
}

enum HighlightKind {
    ClutchClear,        // node cleared with < 3s remaining
    MassDestruction,    // 5+ cells destroyed in a single shockwave/chain
    PerfectStreak,      // N perfect bumps in a row
    FastClear,          // node cleared in < N seconds
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

## Acceptance Criteria

1. RunStats accumulates throughout the run via message observation
2. Run-end screen displays all tracked stats
3. Chip build summary shows what was collected and at what stacks
4. Flux earned is calculated and displayed
5. Seed is displayed on run-end screen
6. Retry option starts a new run with the same seed
7. At least 3 highlight moment types are detected and displayed on the run-end screen
