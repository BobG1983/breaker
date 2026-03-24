---
name: resolved_checker_bugs
description: Past scenario runner bugs that caused incorrect invariant results — fixed, kept as regression reference. Also documents confirmed flaky scenarios.
type: project
---

# Resolved Checker Bugs

Past bugs in the scenario runner's invariant checkers that caused incorrect results.
All resolved. Kept as reference to prevent regression.

## BoltInBounds — bottom boundary check (RESOLVED)

**Bug:** Checker flagged bolts exiting through the open bottom (life-loss mechanic).
**Root cause:** Bottom boundary was checked, but the game has no floor wall by design.
**Fix:** Removed bottom boundary check entirely from `check_bolt_in_bounds`.
**File:** `breaker-scenario-runner/src/invariants/checkers/bolt_in_bounds.rs`

## ValidBreakerState — Braking → Dashing ordering race (RESOLVED)

**Bug:** Checker sampled breaker state before `update_breaker_state` ran, then on the
next frame saw the final state after a double-transition (`Braking → Settling → Idle → Dashing`),
recording an illegal `Braking → Dashing` transition.
**Root cause:** No ordering constraint between checker and `update_breaker_state`.
**Fix:** Added `.after(update_breaker_state)` to invariant block in lifecycle plugin.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## TimerMonotonicallyDecreasing — same-duration node transitions (RESOLVED)

**Bug:** When consecutive nodes had the same timer duration, `remaining` jumped back near
`total` on the new node but `total` didn't change, so the checker saw an illegal increase.
**Root cause:** Checker only detected node transitions via `total` change.
**Fix:** Added `near_total` detection — if `remaining` jumps to within 1.0 of `total`,
treat it as a node transition reset.
**File:** `breaker-scenario-runner/src/invariants/checkers/timer_monotonically_decreasing.rs`

## NoEntityLeaks — baseline sampled during Loading (RESOLVED)

**Bug:** Baseline entity count was sampled at a fixed frame, which could be during
`GameState::Loading` when few entities existed. Post-Loading entity spawns looked like leaks.
**Root cause:** Frame-based baseline sampling, not gameplay-aware.
**Fix:** Baseline now sampled on `SpawnNodeComplete` message (all gameplay entities spawned).
**File:** `breaker-scenario-runner/src/invariants/checkers/no_entity_leaks.rs`

## BreakerPositionClamped — checker ran before enforce_frozen_positions (RESOLVED)

**Bug:** In self-test scenarios with `disable_physics: true`, the checker could run before
`enforce_frozen_positions` restored the teleported position, seeing the clamped position instead.
**Root cause:** Invariant block was an unordered tuple — no guaranteed execution order.
**Fix:** Changed invariant block to `.chain()` with `enforce_frozen_positions` first.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## Invariant checkers firing during Loading state (RESOLVED)

**Bug:** Under parallel I/O contention, checkers ran before `GameState::Playing` was entered,
finding uninitialized or missing entities and producing spurious violations.
**Root cause:** Invariant block had no `run_if` guard for gameplay readiness.
**Fix:** Added `.run_if(|stats| stats.entered_playing)` to invariant block.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## tick_scenario_frame counting during Loading (RESOLVED)

**Bug:** Frame counter ticked from app startup. In parallel mode, frame_mutations at
frame 30 were missed because Playing wasn't entered yet.
**Root cause:** `tick_scenario_frame` had no `entered_playing` gate.
**Fix:** Added `.run_if(entered_playing)` to `(tick_scenario_frame, check_frame_limit).chain()`.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`

## entered_playing race condition — SpawnNodeComplete fix (RESOLVED 2026-03-23)

**Bug:** `entered_playing` was set by `tag_game_entities` (OnEnter(Playing)), but bolt/breaker
entities hadn't been spawned yet under parallel I/O load at that moment. Three self-test
scenarios were flaky: `bolt_count_exceeded`, `timer_increase`, `physics_frozen_during_pause`.
**Root cause:** OnEnter(Playing) fires before spawn systems flush deferred commands. Tags were
applied to entities that didn't exist yet, so `bolts_tagged=0 / breakers_tagged=0` even with
`entered_playing=true`, causing health-check failures in the verdict evaluator.
**Fix:** Moved `entered_playing` assignment from `tag_game_entities` to a new system
`mark_entered_playing_on_spawn_complete` that reads the `SpawnNodeComplete` message. Frame
counting and invariant checking now only begin once all entities are actually spawned.
**File:** `breaker-scenario-runner/src/lifecycle/mod.rs`
**Side effect:** Three unit tests in `lifecycle/tests.rs` are now failing (see below — known
failing tests). They were written against the old behavior and need to be updated.

---

## Confirmed Flaky Scenarios — RESOLVED by SpawnNodeComplete fix (2026-03-23)

The three scenarios below were formerly flaky under `--all` parallel mode. The
`entered_playing` → `SpawnNodeComplete` migration resolved all three. They now pass
reliably in both `--all` and individual runs.

### bolt_count_exceeded — formerly FLAKY, now STABLE

**Scenario:** `scenarios/self_tests/bolt_count_exceeded.scenario.ron`
**Former failure:** `entered_playing=false` race — game didn't reach Playing within frame budget.
**Status:** RESOLVED. All 47 scenarios including this one passed on 2026-03-23 `--all` run.

### timer_increase — formerly FLAKY, now STABLE

**Scenario:** `scenarios/self_tests/timer_increase.scenario.ron`
**Former failure:** `bolts=0 breakers=0` despite `entered_playing=true` — entities not yet spawned
when OnEnter(Playing) fired under I/O contention.
**Status:** RESOLVED. Passed reliably on 2026-03-23 `--all` run.

### physics_frozen_during_pause — formerly FLAKY, now STABLE

**Scenario:** `scenarios/self_tests/physics_frozen_during_pause.scenario.ron`
**Former failure:** invariant never fired — frame mutation window compressed under parallel I/O load.
**Status:** RESOLVED. Passed reliably on 2026-03-23 `--all` run.

---

## Known Failing Unit Tests (require test updates — NOT game bugs)

As of 2026-03-23, three unit tests in `breaker-scenario-runner/src/lifecycle/tests.rs` fail
because they were written against the old behavior where `tag_game_entities` set
`ScenarioStats::entered_playing = true`. That responsibility moved to
`mark_entered_playing_on_spawn_complete` (reads `SpawnNodeComplete` message).

### Failing tests

1. `lifecycle::tests::scenario_stats_entered_playing_set_by_tag_game_entities` (line 791)
   - Asserts `entered_playing == true` after running `tag_game_entities` alone.
   - `tag_game_entities` no longer sets `entered_playing`, so the assertion fails.
   - Fix: rename to `scenario_stats_entered_playing_set_by_mark_entered_playing_on_spawn_complete`
     and rewrite to send a `SpawnNodeComplete` message and assert the new system sets the flag.

2. `lifecycle::tests::check_bolt_in_bounds_is_registered_in_scenario_lifecycle` (line 191)
   - Uses `lifecycle_test_app()` which includes `ScenarioLifecycle`. The invariant block is
     gated on `entered_playing`. No `SpawnNodeComplete` message is ever sent in the test, so
     `entered_playing` stays `false`, invariants never run, ViolationLog stays empty.
   - Fix: send a `SpawnNodeComplete` message before the tick (or manually set
     `ScenarioStats::entered_playing = true` in the test world before ticking).

3. `lifecycle::tests::check_no_nan_is_registered_in_scenario_lifecycle` (line 229)
   - Same root cause as #2 — `entered_playing` never becomes `true`.
   - Fix: same approach as #2.

All three tests are in `breaker-scenario-runner/src/lifecycle/tests.rs`.
Route to writer-tests with a test revision spec once confirmed. Confidence: high.

---

## Unresolved Game Bugs (correctly detected by scenario runner)

### BoltInBounds — prism_scatter sustained violation

**Scenario:** Prism + Scatter layout + Chaos(seed=47, action_prob=0.3)
**Violation:** BoltInBounds x1570 frames 6830..7689
**Status:** Real game physics bug. Multiple extra bolts simultaneously outside playfield
bounds for sustained period. Root cause not yet confirmed — likely degenerate reflection
state in bolt physics under Scatter layout with many cells destroyed.
**This is NOT a scenario runner bug.** The runner correctly detects it.
