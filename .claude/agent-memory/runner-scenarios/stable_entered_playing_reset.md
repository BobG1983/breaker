---
name: entered_playing reset pattern
description: ScenarioStats::entered_playing must be reset to false on run restart; failure causes BreakerCountReasonable false positives during teardown gaps in allow_early_end: false scenarios
type: project
---

## Pattern: reset entered_playing on run restart

`ScenarioStats::entered_playing` is set `true` by `mark_entered_playing_on_spawn_complete` when `SpawnNodeComplete` fires.

It must be reset to `false` in `restart_run_on_end` (in `breaker-scenario-runner/src/lifecycle/systems/frame_control.rs`) when the run restarts. Without this reset, invariant checkers (including `BreakerCountReasonable`) run during the 6-frame gap between `PrimaryBreaker` despawn and the next `SpawnNodeComplete` — producing count=0 false positives.

**Fixed:** 2026-04-06 in feature/scenario-runner-wiring. All 116 scenarios pass.

**How to apply:** If `BreakerCountReasonable` fires in a pattern of "count=0 for ~6 frames at regular restart intervals" in `allow_early_end: false` scenarios, check that `restart_run_on_end` resets `entered_playing = false`.
