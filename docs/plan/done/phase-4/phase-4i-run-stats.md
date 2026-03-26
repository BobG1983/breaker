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
    ClutchClear,            // node cleared with < N secs remaining (configurable)
    MassDestruction,        // M+ cells destroyed within a time window (configurable)
    PerfectStreak,          // K+ consecutive perfect bumps (configurable)
    FastClear,              // node cleared in < fraction of allotted time (configurable)
    FirstEvolution,         // first chip evolution in the run
    NoDamageNode,           // node cleared without losing a bolt
    MostPowerfulEvolution,  // evolution that dealt the most total damage
    CloseSave,              // bolt saved by bump within N pixels of bottom (configurable)
    SpeedDemon,             // node cleared in < N seconds wall-clock (configurable)
    Untouchable,            // N+ consecutive no-damage nodes (configurable)
    ComboKing,              // N+ cells destroyed between breaker hits (configurable)
    PinballWizard,          // N+ cell bounces without breaker contact (configurable)
    Comeback,               // cleared despite losing N+ bolts (configurable)
    PerfectNode,            // every bump in the node was perfect grade
    NailBiter,              // node cleared while bolt within N pixels of bottom (configurable)
}
// All thresholds are loaded from defaults.highlights.ron via HighlightDefaults → HighlightConfig.
```

### Message Observation Systems

Systems that listen to existing messages and increment `RunStats`:
- `CellDestroyed` → `cells_destroyed += 1`
- `BumpPerformed { grade }` → `bumps_performed += 1`, if perfect: `perfect_bumps += 1`
- `BoltLost` → `bolts_lost += 1`
- `ChipSelected { name }` → `chips_collected.push(name)`
- `NodeCleared` → `nodes_cleared += 1` (via `track_node_cleared_stats`)
- Time tracking: `track_time_elapsed` accumulates `time.delta_secs()` each FixedUpdate tick during `PlayingState::Active`

No new mutation paths — purely observational.

### Highlight Detection Systems (10 systems in run domain)

All detection systems emit `HighlightTriggered { kind }` on every trigger (for juice) and additionally push to `RunStats::highlights` only once per kind per run, subject to `HighlightConfig::highlight_cap`. Implemented as separate systems:

- `detect_mass_destruction` — reads `CellDestroyed`, maintains sliding time window
- `detect_close_save` — reads `BumpPerformed`, queries bolt `Transform` vs `PlayfieldConfig::bottom()`
- `detect_combo_and_pinball` — reads `CellDestroyed`, `BoltHitCell`, `BoltHitBreaker`; dual detection in one system
- `detect_nail_biter` — reads `NodeCleared`, queries nearest bolt distance from bottom
- `detect_first_evolution` — reads `ChipSelected`, queries `EvolutionRegistry` for name match; also increments `evolutions_performed`
- `track_node_cleared_stats` — reads `NodeCleared`; detects `ClutchClear`, `NoDamageNode`, `FastClear`, `PerfectStreak`, `SpeedDemon`, `Untouchable`, `Comeback`, `PerfectNode` on node clear
- `track_bumps` — reads `BumpPerformed`; increments `bumps_performed`/`perfect_bumps`; detects `PerfectStreak`-related tracker updates

### HighlightDefaults / HighlightConfig RON Pipeline

All thresholds are configurable. `HighlightDefaults` (`run/definition.rs`) is a `GameConfig`-derived asset type with a corresponding `assets/config/defaults.highlights.ron` file. The `GameConfig` macro generates `HighlightConfig` resource with `From<HighlightDefaults>`. Detection systems all read `Res<HighlightConfig>`. `RunPlugin` initializes `HighlightConfig` via `init_resource` using the `Default` impl (which matches the RON file values). The RON file is not wired into `DefaultsCollection` for hot-reload but the defaults are correctly reflected at runtime.

### Death Copy Subtitles

Run-end screen picks a seed-deterministic subtitle per outcome from 5 variants each:
- `RunOutcome::Won` — 5 flavour strings (e.g., "The bolt obeys. For now.")
- `RunOutcome::TimerExpired` — 5 flavour strings
- `RunOutcome::LivesDepleted` — 5 flavour strings
- `RunOutcome::InProgress` — falls back to plain "RUN ENDED" with no subtitle

Subtitle index is `seed % 5`.

### Run-End Screen Enhancement

Current run-end screen shows win/lose. Enhance with:

- **Outcome**: WIN / LOSE (prominent)
- **Stats grid**: Nodes cleared, cells destroyed, bumps, perfect bumps, bolts lost, time elapsed
- **Build summary**: List of chips collected (with stack counts), evolutions performed
- **Flux earned**: Calculated from stats (formula TBD — even if Flux spending isn't built yet, show the earned amount)
- **Seed**: Displayed for sharing/replay
- **Highlights**: Up to N highlight moments from the run, where N = `HighlightConfig::highlight_cap` (default 5, RON-configurable). Falls back to 3 if `HighlightConfig` is absent.
- **Actions**: Press Enter to continue (Retry/New Run/Main Menu are future work)

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
7. All 15 `HighlightKind` variants are detected by the 10 detection systems and can appear on the run-end screen
8. All highlight thresholds are configurable via `defaults.highlights.ron` (no hardcoded constants)
9. Seed-based death copy subtitles are displayed per run outcome (5 variants each, `Won`, `TimerExpired`, `LivesDepleted`)
