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

---

## Confirmed Flaky Scenarios (parallel I/O contention only)

These scenarios fail non-deterministically under `--all` parallel mode but pass reliably
when run individually with `-s <name>`. They are NOT game bugs.

### bolt_count_exceeded — self-test (FLAKY under --all)

**Scenario:** `scenarios/self_tests/bolt_count_exceeded.scenario.ron`
**Failure signature (--all mode):**
```
frames=120 violations=0 bolts=0 breakers=0 entered_playing=false
expected 1 violation(s) matching [BoltCountReasonable], found 0
no bolts were tagged — bolt invariants are vacuous
no breakers were tagged — breaker invariants are vacuous
```
**Root cause:** `entered_playing=false` — game did not reach `GameState::Playing` within the
120-frame budget. Under high parallelism, I/O delays during asset loading cause the scenario to
exhaust its frame count before `bypass_menu_to_playing` fires. Since the invariant block is
gated on `entered_playing`, no violations are recorded, so the `expected_violations` check fails.
**Confirmed individual pass:** Yes (two consecutive individual runs both pass).
**This is NOT a game bug.** It is a timing artifact of running 45+ scenarios simultaneously.

### timer_increase — self-test (FLAKY under --all)

**Scenario:** `scenarios/self_tests/timer_increase.scenario.ron`
**Failure signature (--all mode):**
```
frames=120 violations=2 bolts=0 breakers=0 entered_playing=true
TimerMonotonicallyDecreasing   x1   frame 30
no bolts were tagged — breaker invariants are vacuous
no invariant checks ran — game loop may not have executed
```
**Root cause:** `bolts=0 breakers=0` despite `entered_playing=true` — the `tag_game_entities`
system ran on `OnEnter(GameState::Playing)` but at that moment the bolt and breaker entities
hadn't yet been spawned (asset loading was still in progress under I/O contention). The
`SetTimerRemaining(80.0)` frame mutation at frame 30 fires (explaining the x1 violation), but
without tagged entities the "no bolts were tagged" failure condition triggers.
**Confirmed individual pass:** Yes (two consecutive individual runs both pass).
**This is NOT a game bug.** It is a timing artifact of parallel asset loading.

### physics_frozen_during_pause — self-test (FLAKY under --all)

**Scenario:** `scenarios/self_tests/physics_frozen_during_pause.scenario.ron`
**Failure signature (--all mode):**
```
frames=120 actions=0 violations=0 logs=0 bolts=1 breakers=1 entered_playing=true
expected violation PhysicsFrozenDuringPause never fired
```
**Root cause:** Unlike `bolt_count_exceeded` / `timer_increase` where `entered_playing=false` reveals the I/O
delay, here `entered_playing=true` and `bolts=1` — entities are tagged. However the `TogglePause` frame
mutation at frame 30 + `MoveBolt` at frame 35 combination apparently does not produce a sampled position
delta across consecutive `check_physics_frozen_during_pause` ticks under high parallelism. The checker stores
previous-position in a `Local` keyed per-entity; under I/O contention the fixed-update tick ordering may
compress or skip the frame window where both a "pre-move" and "post-move" tick are observed while in Paused
state. Net result: invariant never fires, `expected_violations` check fails.
**Confirmed individual pass:** Yes (two consecutive individual runs both pass with `violations=2`).
**This is NOT a game bug.** It is a timing artifact of parallel fixed-update scheduling under high I/O load.

---

## Unresolved Game Bugs (correctly detected by scenario runner)

### BoltInBounds — prism_scatter sustained violation

**Scenario:** Prism + Scatter layout + Chaos(seed=47, action_prob=0.3)
**Violation:** BoltInBounds x1570 frames 6830..7689
**Status:** Real game physics bug. Multiple extra bolts simultaneously outside playfield
bounds for sustained period. Root cause not yet confirmed — likely degenerate reflection
state in bolt physics under Scatter layout with many cells destroyed.
**This is NOT a scenario runner bug.** The runner correctly detects it.
