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

**Why:** Scenario runner is diagnostic tooling. It runs once per test invocation, not continuously. Per-frame allocations in visual mode only; headless mode (the common path) still has the HashSet allocation.
