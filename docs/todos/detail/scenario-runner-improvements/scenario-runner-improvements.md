# Scenario Runner Improvements

## Summary
Three-part scenario runner investment: (A) verbose log, screenshots, visual mode, and streaming execution; (B) coverage gaps — new invariants, new scenarios, fix stale checkers; (C) performance audit — trim frame counts, evaluate parallel-race scenarios, reduce run time.

## Part A: Verbose Log + Visual Mode (formerly todo #9)

Five features in implementation order:

1. **Verbose violation log file** — structured output directory at `/tmp/breaker-scenario-runner/YYYY-MM-DD/N/violations.log`, path printed in `print_summary`
2. **`--clean` flag** — deletes `/tmp/breaker-scenario-runner/` recursively, exits immediately
3. **First-failure screenshots** — one screenshot per `(scenario_name, InvariantKind)` pair via Bevy screenshot API, requires `--visual` mode
4. **Resolution-independent rendering + window tiling** — `sync_ui_scale` system using `UiScale` resource to scale all `Val::Px` values globally (see researched fix below), grid tiling for `--visual` parallel runs, pocket reuse
5. **Streaming execution** — queue-based pool of N concurrent slots (from `-p` flag), no wave-boundary waits, pairs with tiling for pocket reuse

### UI Scaling Fix (Researched — Ready)

Root cause: camera uses `ScalingMode::AutoMin` but UI resolves `Val::Px` against physical pixels. Fix is a single `sync_ui_scale` system:

```rust
fn sync_ui_scale(
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.single() {
        let scale = (window.width() / 1920.0).min(window.height() / 1080.0);
        ui_scale.0 = scale;
    }
}
```

See research files for details:
- [UI scaling investigation](research/ui-scaling-investigation.md)
- [Bevy UI scaling patterns](research/bevy-ui-scaling.md)

### Reference Files (Part A)
- `breaker-scenario-runner/src/main.rs` — entry point, `Args` struct (line 196), CLI parsing via clap `#[derive(Parser)]`
- `breaker-scenario-runner/src/runner/execution.rs` — parallel execution engine:
  - `print_summary()` (line 138) — current summary output
  - `run_all_parallel()` (line 284) — orchestrates subprocess batching
  - `spawn_batched()` (line 191) — spawns subprocess waves via `specs.chunks(parallelism)`, waits per batch
  - `ChildResult` (line 176) — captures stdout/stderr/exit code from subprocesses
  - `StressResult` / `StressFailure` (line 333) — aggregates stress scenario results
- Subprocess communication: stdout/stderr piped + exit code, parent reads via `wait_with_output()`

### Dependencies (Part A)
- Feature 1 (log): no deps
- Feature 2 (--clean): depends on Feature 1 (directory structure)
- Feature 3 (screenshots): depends on Feature 1 (output dir), requires `--visual`
- Feature 4 (tiling): depends on UI scaling fix
- Feature 5 (streaming): independent of 1-3, pairs with tiling

## Part B: Scenario Coverage Gaps (formerly todo #4)

Identified during Full Verification Tier review of `feature/breaker-builder-pattern` (2026-04-02).

### HIGH Priority

1. **`BreakerPositionClamped` invariant stale after size boost refactor** — doesn't account for `ActiveSizeBoosts`, produces false positives. Fix: query `ActiveSizeBoosts` and use effective half-width.

2. **No scenario exercises non-trivial `entity_scale`** — all layouts use `entity_scale: 1.0`. Fix: create `node_scale_entity_chaos.scenario.ron` with `entity_scale: 0.5`, apply `SizeBoost(2.0)` on every Bumped.

### MEDIUM Priority

3. **No multi-node scenario for breaker reuse path** — reuse path only in unit tests. Fix: add `BreakerCountReasonable` invariant, create multi-node scenario with `allow_early_end: true`.

4. **`entity_scale_collision_chaos` only exercises boost, not node scaling** — layout uses `entity_scale: 1.0`. Fix: create `bolt_radius_clamping_chaos.scenario.ron` with explicit `MinRadius`/`MaxRadius`.

5. **`sync_breaker_scale` height boost has no scenario regression** — Fix: enable `BreakerPositionClamped` and `AabbMatchesEntityDimensions` in existing scenarios when size boost is active.

6. **No scenario exercises pause/quit path** — `toggle_pause` reads `ButtonInput<KeyCode>` directly, not `InputActions`. Fix: add `QuitToMenu` meta-action, fix or remove `aegis_pause_stress` scenario.

### New Invariants Needed
- **`BreakerCountReasonable`** — fires if `With<PrimaryBreaker>` count != 1 during `GameState::Playing`
- **`BoltScaleCoherent`** (optional) — checks scale matches effective radius
- **Update `BreakerPositionClamped`** — account for `ActiveSizeBoosts`

### Reference Files (Part B)

**Scenario runner crate** (`breaker-scenario-runner/`):
- `src/types/definitions/invariants.rs` — `InvariantKind` enum (line 6), `ALL` constant (line 61), `fail_reason()` (line 90). Add new variants here.
- `src/invariants/checkers/mod.rs` — checker module registry. Each checker is a Bevy system (not trait-based) that appends `ViolationEntry` to `ViolationLog` resource.
- `src/invariants/checkers/breaker_position_clamped.rs` — `check_breaker_position_clamped()` (line 11). Fix this for size boost awareness.
- `src/invariants/checkers/no_nan.rs` — `check_no_nan()` (line 18). Simple example of the checker pattern.
- `src/invariants/checkers/bolt_in_bounds/checker.rs` — `check_bolt_in_bounds()` (line 21). More complex example.

**Scenario RON files** (`breaker-scenario-runner/scenarios/`):
- Subdirs: `mechanic/`, `chaos/`, `stress/`, `self_tests/`, `regressions/`
- Example: `scenarios/mechanic/aegis_dash_wall.scenario.ron` — shows `breaker`, `layout`, `input`, `max_frames`, `disallowed_failures`, `allowed_failures`, `debug_setup` fields

**Game crate components** (`breaker-game/src/`):
- `effect/effects/size_boost.rs:7` — `ActiveSizeBoosts`
- `shared/components.rs:7` — `BaseWidth`
- `shared/components.rs:34` — `NodeScalingFactor`
- `breaker/components/core.rs:21` — `PrimaryBreaker`
- `input/resources.rs:59` — `InputActions`

## Part C: Performance Audit (formerly todo #8)

Reduce stress scenario frame counts, evaluate which parallel-race-condition scenarios are still needed, trim total run time (currently 10+ min).

**[NEEDS DETAIL]** — specifics TBD. Should be done after Parts A and B so we're auditing the final scenario set.

## Scope
- In: all scenario runner tooling, all scenario coverage, all scenario performance
- Out: game logic changes (except invariant checker updates), non-scenario testing

## Dependencies
- Depends on: nothing
- Blocks: nothing directly, but coverage gaps (Part B) improve confidence for all future work

## Status
`ready` (Parts A and B are fully scoped; Part C needs detail but is last in sequence)
