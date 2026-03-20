---
name: known_invariant_false_positives
description: Invariant checkers that fire on valid gameplay — root causes and fix directions
type: project
---

# Known Invariant False Positives

These invariants fire on valid/intentional gameplay. Confirmed by source reading
and verbose scenario output.

## BoltInBounds — fires on every life-loss event

**Root cause:** No floor wall exists by design (`spawn_walls.rs` spawns only
left, right, ceiling). The bolt exits through the bottom to model life loss.
The invariant threshold (`bottom - radius - 1`) and `bolt_lost` threshold
(`bottom - radius`) are 1 px apart. The invariant fires once per life-loss event
at the single frame when the bolt is between -554 and -555 (for 14px radius, 60px
half-height playfield).

**Every scenario** that includes `BoltInBounds` will fail on any scenario that
allows bolt/life-loss mechanics. All 21 failing scenarios are affected.

**Fix direction:** Exclude the bottom wall check from `check_bolt_in_bounds`
in `breaker-scenario-runner/src/invariants.rs:251`, OR raise the margin to
exactly `radius` (not `radius + 1`) to match the bolt_lost threshold.

---

## ValidBreakerState — Dashing → Settling is a legal transition (FIXED)

This is already resolved: `check_valid_breaker_state` lists `(Dashing, Braking | Settling)` as legal.

---

## ValidBreakerState — Braking → Dashing appears illegal due to system ordering race (NEW, 2026-03-19)

**Invariant fires:** `prism_scatter_stress` copy 25/32, frame 3257
**Scenario:** Prism + Scatter + Chaos(seed=47, action_prob=0.3)

**Root cause:** No explicit ordering between `check_valid_breaker_state` (invariant
checker group, before `PhysicsSystems::BoltLost`) and `update_breaker_state` (after
`BreakerSystems::Move`). `handle_idle_or_settling` in `dash.rs` performs TWO state
transitions in one call: timer expiry gives `Settling → Idle`, then dash input immediately
gives `Idle → Dashing`. Both happen synchronously with no frame boundary.

When the checker samples before `update_breaker_state` on frame N (stores `Braking`),
then `update_breaker_state` runs (`Braking → Settling`), then on frame N+1 the checker
samples after `update_breaker_state` (which ran `Settling → Idle → Dashing` in one call),
the checker records: previous=`Braking`, current=`Dashing` → `Braking → Dashing` is not
in the legal transition set → VIOLATION.

**Intermittency:** 1/32 copies fail (stress parallelism causes non-deterministic scheduling).
Does not fire in single-scenario (`-s`) mode or the non-stress `prism_scatter` scenario.

**Fix direction:** Add explicit ordering in lifecycle/mod.rs:
`check_valid_breaker_state.after(update_breaker_state)` (or after a new
`BreakerSystems::StateMachine` set that `update_breaker_state` is placed in).

**Confidence: HIGH** — full code path traced:
`breaker-game/src/breaker/systems/dash.rs:110-146` (double-transition in single call),
`breaker-scenario-runner/src/lifecycle/mod.rs:149-177` (checker ordering),
`breaker-game/src/breaker/plugin.rs:46-68` (no mutual ordering between checker and state machine)

---

## TimerMonotonicallyDecreasing — same-duration node transitions not detected

**Root cause:** The fix tracks `(remaining, total)` and resets tracking when
`total` changes. But all nodes in Aegis/Chrono/Prism use the same timer
duration (60s), so `total` does not change across node transitions. The
invariant fires on the first tick of a new node when `remaining` resets from
~40-54s back to 59.984s (60s minus one fixed tick).

**Fix direction:** Instead of detecting node transitions via `total` change,
add a node generation counter (new `NodeGeneration` resource that increments
on each `OnEnter(Playing)`), and track `(remaining, generation)` in the Local.
Alternatively: reset tracking whenever `current > prev_remaining` — simpler
but loses subtle increase-bug detection.

See: `breaker-scenario-runner/src/invariants.rs:86-117`,
`breaker-game/src/run/node/systems/tick_node_timer.rs:36-38`

---

## NoEntityLeaks — fires in parallel subprocess mode when assets load slowly

**Root cause:** `check_no_entity_leaks` samples the entity count at fixed-update
frame 60 as baseline. In parallel subprocess mode (`--all`, 27 processes on macOS
under I/O contention), `bevy_asset_loader` can take >60 fixed-update frames to
finish loading RON assets. The baseline is captured in `GameState::Loading` with
only ~7 system entities. After `Playing` is entered (post-frame-60), ~49 game
entities spawn. Every 120-frame check then fires `count=49 > baseline×2=14`.

In `--serial` and single-scenario (`-s`) mode this does not occur — assets load
within frame 60 reliably when only one process has disk I/O.

**Observed violation message:** `NoEntityLeaks FAIL frame=120 count=49 baseline=7 (>14 threshold)`

**Fix direction:** Defer baseline sampling until `GameState::Playing` has been
entered. In `check_no_entity_leaks` (`invariants.rs:448-477`), add a check for
`stats.entered_playing` (or read `GameState` resource) before setting the
baseline, rather than using a fixed frame-60 sample.

**Confidence: HIGH**

---

## BoltInBounds — fires in parallel subprocess mode under I/O contention (two manifestations)

**Also caused by the same asset-loading race as NoEntityLeaks above.**

**Manifestation A — self-test scenario fails health checks (bolt_oob_detection):**
When asset loading races in a subprocess, `ScenarioTagBolt` / `ScenarioTagBreaker` are never
attached (the lifecycle plugin hasn't run tagging yet). The verdict health checks report
`bolts=0, breakers=0` and fire "no bolts were tagged", "no invariant checks ran", and
"expected violation BoltInBounds never fired" (since `check_bolt_in_bounds` found no tagged
entities). Passes reliably with `-s bolt_oob_detection`.

**Manifestation B — stress scenario fires real BoltInBounds (prism_concurrent_hits, chrono_penalty_stress):**
Under I/O contention, asset loading delays push game initialization into a transient state
where bolt physics simulate briefly before all walls/layout are ready. Bolts escape the
playfield boundary during this window. Violations are sustained for hundreds of frames
(e.g. x2420 frames 13613..15000, x3729 frames 1232..3207). Always passes with `-s <name>`.

**Non-deterministic:** A different scenario fails on each `--all` run. All pass when run
individually. Confirmed across 3 separate `--all` runs on 2026-03-19.

**Fix direction (same as NoEntityLeaks):** Defer scenario lifecycle initialization until
`GameState::Playing` has been confirmed entered; do not allow invariant checks to run
during the `Loading` phase when entities may not yet exist.

---

## BoltInBounds — prism_scatter sustained violation (pre-existing, unrelated to asset race)

**First observed:** 2026-03-19 (Local<Vec> refactor run)
**Violation:** BoltInBounds x1570 frames 6830..7689 (~1.8/frame for 859 frames)
**Scenario:** Prism + Scatter layout + Chaos(seed=47, action_prob=0.3)

Multiple extra bolts simultaneously outside playfield bounds for sustained period.
The `Local<Vec>` change in `bolt_lost` / `handle_cell_hit` is NOT the cause — both
systems call `.clear()` at frame start (fully equivalent to per-frame allocation).

Root cause not yet confirmed. Working hypothesis: under Scatter layout with many cells
destroyed, extra Prism bolts encounter a degenerate reflection state (velocity
compounding or wall-exit without `bolt_lost` triggering). Unique to Scatter + Prism +
seed 47; does not appear in prism_stress, prism_fortress, prism_concurrent_hits,
prism_accumulation, or prism_bolt_stabilization.

**Confidence: MEDIUM** — physics root cause not yet read; `bolt_cell_collision` and
wall-reflection math in `shared::math` are the most likely suspects.

---

## BreakerPositionClamped — never fires in breaker_oob_detection self-test (NEW, 2026-03-19)

**Scenario:** `breaker_oob_detection` — Aegis + Corridor, breaker teleported to x=2000.0 via `debug_setup`, `disable_physics=true`. Expects BOTH `BreakerInBounds` AND `BreakerPositionClamped` to fire.

**Observed:** `BreakerInBounds` fires 105 times (frames 16-120). `BreakerPositionClamped` never fires. Fails with `-s` (single-run), confirming it is a real bug not a parallelism artifact.

**Root cause:** `check_breaker_position_clamped` queries `(Entity, &Transform, &BreakerWidth), With<ScenarioTagBreaker>`. `BreakerWidth` is inserted by `init_breaker_params` which uses `Commands` (deferred). The entity is spawned in `spawn_breaker` (also deferred). In `OnEnter(GameState::Playing)`, there is no explicit `ApplyDeferred` between `spawn_breaker` and `init_breaker_params` in the breaker plugin's schedule, so when `init_breaker_params` runs, the newly-spawned `Breaker` entity is not yet visible via query — the deferred spawn has not been applied. As a result, `init_breaker_params` is a no-op on the first node, `BreakerWidth` is never inserted, and the `check_breaker_position_clamped` query matches no entities.

**Why BreakerInBounds still fires:** `check_breaker_in_bounds` queries only `(Entity, &Transform), With<ScenarioTagBreaker>` — no `BreakerWidth` required. `ScenarioTagBreaker` is inserted by `tag_game_entities`, which runs BEFORE `apply_debug_setup` in the scenario lifecycle chain. The `Transform` teleport to x=2000.0 happens in `apply_debug_setup`. So the entity IS tagged and has the right Transform by the first `FixedUpdate`.

**Evidence:** `spawn_breaker.rs:37-54` — no `BreakerWidth` in the spawn bundle. `init_breaker_params.rs:27-30` — inserts `BreakerWidth` via `Commands`, filtered by `Without<BreakerMaxSpeed>`, deferred. `breaker/plugin.rs:36-44` — no `ApplyDeferred` between `spawn_breaker` and `init_breaker_params`.

**Fix direction:** Add `ApplyDeferred` between `spawn_breaker` and `init_breaker_params` in `breaker/plugin.rs:36-44`. This allows `init_breaker_params` to find the newly-spawned entity on the first node.

**Confidence: HIGH**

---

## Note: `invariants` field in RON is documentation-only

All invariant check systems run for every scenario unconditionally. The
`invariants: [...]` field in `.scenario.ron` files does NOT filter which
checks are active. Confirmed by reading `lifecycle.rs:138-165`.
