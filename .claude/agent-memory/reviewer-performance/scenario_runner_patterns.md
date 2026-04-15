---
name: Scenario runner performance patterns
description: Confirmed patterns in breaker-scenario-runner that look expensive but are acceptable for diagnostic tooling
type: project
---

This crate is diagnostic tooling, not gameplay. Performance standards are relaxed.

**capture_violation_screenshots** (`src/invariants/screenshot.rs`):
- Runs in `Last` every frame, gated by `resource_exists::<ScreenshotOutputDir>` (only in visual mode).
- `detect_new_violations` allocates a `HashSet<InvariantKind>` every frame even when there are no new violations.
- The early-return on `new_kinds.is_empty()` executes AFTER the HashSet is already built.
- InvariantKind::ALL has 23 variants — ViolationLog grows only on actual violations, which are rare in passing scenarios.
- **Minor watch**: HashSet allocation happens every frame in visual mode even when ViolationLog is empty. Could short-circuit before building the set. Not worth fixing — visual mode is rare.

**check_coverage** (`src/coverage.rs`):
- `is_covered` closure does a linear `self_test_names.contains(name)` on a Vec for every scenario * InvariantKind combination.
- Called once at startup/report time, not per-frame. Academic concern.
- `format_coverage_report` calls `report.covered_self_tests.contains(variant)` (linear scan on Vec) for each of 23 variants. Also startup-only.

**tile_config_env_vars** (`src/runner/tiling.rs`):
- Allocates a `Vec<(&str, String)>` per subprocess spawn (2 elements, 2 String allocations for usize→string: `SCENARIO_TILE_INDEX` and `SCENARIO_TILE_COUNT`).
- Called once per subprocess, not per-frame. Negligible.
- `grid_dimensions` recomputed per subprocess (no caching), but it's pure integer arithmetic — free.

**RunLog** (`src/runner/run_log.rs`):
- `write_line` does `line.to_owned()` per call (alloc per log line). Acceptable — log lines are infrequent and this is async write path.
- `flush` creates a one-shot `mpsc::channel` per call — intended for synchronization, not a hot path.
- Background BufWriter thread is correct pattern for async IO.

**check_offering_no_duplicates** (`src/invariants/checkers/check_offering_no_duplicates.rs`):
- Allocates a `HashSet` and calls `.to_owned()` per chip name every FixedUpdate frame while in `ChipSelectState::Selecting`.
- ChipOffers typically has 3 chips max. HashSet is tiny (3 entries). ChipSelect state is brief (1-2 frames in headless).
- Confirmed Minor/Watch: not worth fixing at current scale.

**check_chip_offer_expected** (`src/invariants/checkers/check_chip_offer_expected.rs`):
- Runs in `Update` gated on `in_state(ChipSelectState::Selecting).and(resource_exists::<ChipOffers>)`.
- On violation, collects offer names into a `Vec<_>` and calls `.join()` for the message string — allocation only on actual violation, which is rare.
- Schedule is correct: Update instead of FixedUpdate because auto_skip_chip_select runs in PostUpdate and would race. Intentional design.

**snapshot_eval_data** (`src/runner/app.rs`):
- Runs in `Last` every frame in visual mode. Clones `ViolationLog`, `CapturedLogs`, `ScenarioStats`, `ScenarioDefinition` every frame.
- Only registered in visual mode (headless uses `snapshot_eval_data_from_world` once at end). Cost is bounded by violation count (rare), log count (rare), and definition size (small).
- This is a previous known pattern — skip-per-frame already added for headless (commit f736109b).

**Checker pattern — unconditional `stats.invariant_checks += 1`**:
- Every checker increments `invariant_checks` even when the resource being checked is absent (e.g. NodeTimer, ChipOffers).
- This means all 22 FixedUpdate checkers fire every FixedUpdate frame (minus the playing_gate), but the guards (`let Some(x) = x else { return }`) are extremely cheap — just a None check. (`check_chip_offer_expected` runs in Update separately — 23 total checkers, 22 in FixedUpdate.)
- Confirmed as intentional (commit f736109b: "fix: all invariant checkers increment invariant_checks counter").

**check_chain_arc_count_reasonable** (`src/invariants/checkers/check_chain_arc_count_reasonable.rs`):
- Two separate queries for `ChainLightningChain` and `ChainLightningArc` — each calls `.iter().count()`.
- Combined count done as `chains.iter().count() + arcs.iter().count()`. These can't be merged into one query (different components).
- At current scale: chain arcs are a handful at most. Two `.count()` calls over tiny archetypes is negligible.

**active_invariant_kinds HashSet** (`src/lifecycle/systems/plugin.rs`):
- `active_invariant_kinds()` builds a `HashSet<InvariantKind>` at app build time (in `register_scenario_systems`), not per-frame.
- The HashSet is consumed immediately by `register_active_checkers` and dropped. No per-frame cost.

**apply_tile_layout** (`src/runner/app.rs`):
- Registered in Update schedule, runs every frame in visual mode.
- Guards: `Option<Res<TileConfig>>` early-return (None after first successful apply); `monitors.single()` early-return; `windows.single_mut()` early-return.
- `TileConfig` resource is removed via `commands.remove_resource::<TileConfig>()` after the first successful apply, so the system no-ops from the second frame onward (just the None check on Option<Res<TileConfig>>).
- `monitors` query: `Query<&Monitor, With<PrimaryMonitor>>` — 1 entity, no archetypes concern.
- `windows` query: `Query<&mut Window, With<PrimaryWindow>>` — `&mut Window` where `&Window` would suffice if only resolution is being read, but mutation is needed for `window.resolution = ...` and `window.position = ...`, so mutable is correct.
- Resource removal does NOT cause archetype fragmentation: `TileConfig` is a `Resource`, not a component. Resources are not part of the archetype graph.
- `warn!` macro inside the retry branch: string interpolation happens at the call site, but this only fires while PrimaryMonitor is unavailable (typically 0-1 frames on startup). Negligible.
- **Overall**: clean one-shot pattern. After first apply, cost per frame = Option::is_none() check (a pointer read). No allocations. No archetype concerns. Visual mode only.

**Why:** Scenario runner is diagnostic tooling. It runs once per test invocation, not continuously. Per-frame allocations in visual mode only; headless mode (the common path) still has the HashSet allocation.
