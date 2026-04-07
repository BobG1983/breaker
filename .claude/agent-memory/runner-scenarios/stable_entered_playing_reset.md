---
name: entered_playing never resets on run restart
description: ScenarioStats::entered_playing is set true once and never reset, causing BreakerCountReasonable to fire during run-restart teardown gaps across all allow_early_end: false scenarios
type: project
---

## Root cause: `entered_playing` never resets to `false` on run restart

`mark_entered_playing_on_spawn_complete` (in `breaker-scenario-runner/src/lifecycle/systems/frame_control.rs`) sets `ScenarioStats::entered_playing = true` once, when `SpawnNodeComplete` fires.

It **never resets it to `false`**.

When `allow_early_end: false`, `restart_run_on_end` triggers `RunState::Teardown`. This fires `CleanupOnExit<RunState>`, which despawns the `PrimaryBreaker` entity. The new run then takes ~6 frames to reach a new `SpawnNodeComplete`.

During those ~6 frames, `entered_playing` is still `true`, so all invariant checkers run — including `check_breaker_count_reasonable`, which sees `count=0` and fires.

**Every** scenario with `allow_early_end: false` will show `BreakerCountReasonable` firing at 6-frame windows at regular intervals (one per run-restart cycle).

**Why:** `PrimaryBreaker` has `CleanupOnExit<RunState>` (not `CleanupOnExit<NodeState>`), so it survives node transitions but is despawned on run teardown. The checker fires during the gap.

**Fix location:** `breaker-scenario-runner/src/lifecycle/systems/frame_control.rs`
- `restart_run_on_end` should reset `ScenarioStats::entered_playing = false` when the run restarts
- OR: add an `OnEnter(RunEndState::Active)` or `OnEnter(RunState::Teardown)` system that resets `entered_playing`

**Confirmed:** `multi_node_breaker_reuse` (allow_early_end: true) passes cleanly. All allow_early_end: false scenarios fail with this pattern.

**Violation pattern:** `count=0` for exactly 6 frames per restart cycle, at regular frame intervals.

**Resolution (2026-04-06):** Fixed by resetting `entered_playing = false` inside `restart_run_on_end`. All 116 scenarios pass as of this fix.
