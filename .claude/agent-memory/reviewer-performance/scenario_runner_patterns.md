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
- InvariantKind::ALL has 22 variants — ViolationLog grows only on actual violations, which are rare in passing scenarios.
- **Minor watch**: HashSet allocation happens every frame in visual mode even when ViolationLog is empty. Could short-circuit before building the set. Not worth fixing — visual mode is rare.

**check_coverage** (`src/coverage.rs`):
- `is_covered` closure does a linear `self_test_names.contains(name)` on a Vec for every scenario * InvariantKind combination.
- Called once at startup/report time, not per-frame. Academic concern.
- `format_coverage_report` calls `report.covered_self_tests.contains(variant)` (linear scan on Vec) for each of 22 variants. Also startup-only.

**tile_env_vars** (`src/runner/tiling.rs`):
- Allocates a `Vec<(&str, String)>` per subprocess spawn (4 elements, 4 String allocations for u32→string).
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
- This means all 21 checkers fire every FixedUpdate frame (minus the playing_gate), but the guards (`let Some(x) = x else { return }`) are extremely cheap — just a None check.
- Confirmed as intentional (commit f736109b: "fix: all invariant checkers increment invariant_checks counter").

**check_chain_arc_count_reasonable** (`src/invariants/checkers/check_chain_arc_count_reasonable.rs`):
- Two separate queries for `ChainLightningChain` and `ChainLightningArc` — each calls `.iter().count()`.
- Combined count done as `chains.iter().count() + arcs.iter().count()`. These can't be merged into one query (different components).
- At current scale: chain arcs are a handful at most. Two `.count()` calls over tiny archetypes is negligible.

**active_invariant_kinds HashSet** (`src/lifecycle/systems/plugin.rs`):
- `active_invariant_kinds()` builds a `HashSet<InvariantKind>` at app build time (in `register_scenario_systems`), not per-frame.
- The HashSet is consumed immediately by `register_active_checkers` and dropped. No per-frame cost.

**Why:** Scenario runner is diagnostic tooling. It runs once per test invocation, not continuously. Per-frame allocations in visual mode only; headless mode (the common path) still has the HashSet allocation.
