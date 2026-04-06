# Scenario Runner: Verbose Log + Visual Mode Improvements

## Summary
Five features: (1) verbose violation log file with structured output directory, (2) first-failure screenshots per invariant per scenario, (3) `--clean` flag, (4) visual mode window management (tiling, resolution-independent rendering, pocket reuse), (5) streaming execution (no wave-boundary waits).

## Feature 1: Verbose Violation Log File

Write verbose violation output to a structured directory after all scenarios complete, so detailed violation info is always available without re-running with `-v`.

### Output Directory Structure
```
/tmp/breaker-scenario-runner/
  YYYY-MM-DD/
    1/                          # first run of the day
      violations.log            # verbose violation output
      <scenario>-<frame>-<timestamp>.png   # first-failure screenshots
    2/                          # second run
      ...
```

- Fixed base path: `/tmp/breaker-scenario-runner/`
- Date subdirectory: `YYYY-MM-DD`
- Run number: auto-incrementing within the date directory (scan existing dirs, pick next int)
- `violations.log` contains verbose output for every scenario with violations (including expected violations)
- In `print_summary`, always print: `violation log: /tmp/breaker-scenario-runner/YYYY-MM-DD/N/violations.log`

### Scope
- In: `VerboseViolationLog` accumulation, structured output directory, path printed in `print_summary`
- Out: UI changes, changing the existing stdout output format

## Feature 2: First-Failure Screenshots

When an invariant fails in a scenario, capture a screenshot of the frame that triggered the violation. Only capture the FIRST failure per invariant per scenario run — not every subsequent failure of the same invariant.

### Behavior
- On invariant violation, check if this `(scenario_name, invariant_kind)` pair has already been screenshotted in this run
- If not: save a screenshot to the run directory as `<scenario>-<frame_num>-<timestamp>.png`
- If yes: skip (e.g., BoltInBounds fails 20 times in one scenario → 1 screenshot)
- The screenshot path is included inline in `violations.log` next to the violation entry
- Requires `--visual` mode (headless runs can't screenshot) — in headless mode, log a note like "screenshot unavailable (headless)"

### Deduplication Key
`(scenario_name, InvariantKind)` — one screenshot per unique invariant type per scenario. If `BoltInBounds` and `NoNaN` both fail in the same scenario, that's 2 screenshots.

### Decision: Bevy Screenshot API
Each subprocess captures its own screenshot via Bevy's built-in screenshot API. The output path is passed to the subprocess (e.g., via env var or CLI arg pointing to the run directory). More reliable than OS-level window capture — captures the render buffer directly, no overlapping window issues.

## Feature 3: `--clean` Flag

`cargo scenario -- --clean` deletes the entire `/tmp/breaker-scenario-runner/` directory tree (all dates, all runs, all logs and screenshots).

### Behavior
- Deletes `/tmp/breaker-scenario-runner/` recursively
- Prints: `Cleaned /tmp/breaker-scenario-runner/`
- Exits immediately after cleaning (does not run scenarios)
- Safe to run when no directory exists (no-op with message)

## Feature 4: Resolution-Independent Rendering + Window Tiling

When `--visual` is passed, shrink windows so an entire "wave" of runs fits on screen simultaneously.

### UI Scaling Fix: UiScale Sync System (RESEARCHED — READY)

**Root cause**: the camera uses `ScalingMode::AutoMin { min_width: 1920.0, min_height: 1080.0 }` which scales all world-space entities correctly. But Bevy's UI layout resolves `Val::Px` against physical window pixels, ignoring camera scaling. All 14 UI files use `Val::Px` for padding/gaps/dimensions and absolute `font_size` values designed for 1920×1080.

**Fix**: a single `sync_ui_scale` system — no need to touch the 14 UI files individually:

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

- `UiScale` (in `bevy::prelude`) scales ALL `Val::Px` values and ALL `font_size` values globally
- The `min()` gives letterbox behavior — UI never overflows, may have empty bars on one axis
- Root nodes using `Val::Percent(100.0)` continue to work correctly
- No RON config changes, no per-spawn-system changes, no font size refactors

**Gotchas**:
- `Val::Vw/Vh/VMin/VMax` are NOT scaled by UiScale — but the codebase doesn't use these
- Hairline `Val::Px(1.0)` borders may become sub-pixel at small windows — acceptable for placeholder
- Register in `Update` schedule, no run condition needed (runs in all states)

**This fix is part of this todo** — must be completed before tiling works. See research:
- [UI scaling investigation](research/ui-scaling-investigation.md) — full catalog of all absolute pixel values across 14 files
- [Bevy UI scaling patterns](research/bevy-ui-scaling.md) — UiScale API details, Val variant behavior, font scaling

### Window Tiling
- When `--visual` is passed with parallel runs, tile all windows across the screen
- Layout: fill screen with a grid of windows sized so all are fully visible
- Examples: 1 run = 1 fullscreen window; 32 runs = grid of 32 small windows
- Pocket reuse: when a scenario finishes and its window closes, the next queued scenario spawns in that empty screen slot rather than appearing on top of other windows

### Decision: Scenario name in window title
Each window title shows the scenario name (e.g., "aegis_chaos") for at-a-glance identification.

## Feature 5: Streaming Execution (No Wave Boundary Waits)

Currently, parallel mode runs scenarios in waves — all N scenarios in a wave must finish before the next wave starts. Change this to streaming/queue-based execution:
- Maintain a pool of N concurrent slots (from `-p` flag, e.g., 32)
- When any scenario finishes, immediately start the next queued scenario in its slot
- No waiting for an entire wave to complete
- This pairs with the tiling feature: the finished scenario's window pocket is reused by the next scenario

### Decision: Fixed pool size
Pool size is always determined by the `-p` flag (or default). In visual mode, screen is tiled for N windows. Simple and predictable — no adaptive sizing.

## Dependencies
- Feature 1 (log file): No dependencies
- Feature 2 (screenshots): Depends on Feature 1 (output directory), requires `--visual` mode, uses Bevy screenshot API
- Feature 3 (--clean): Depends on Feature 1 (fixed directory structure)
- Feature 4 (tiling): Depends on UI scaling fix (play area scales, UI doesn't — fix is part of this todo)
- Feature 5 (streaming): Independent of features 1-3, but pairs naturally with tiling for pocket reuse

## Status
`ready` — all features researched and designed. UI scaling fix is a single `sync_ui_scale` system (no 14-file refactor). Screenshot API uses Bevy's `Screenshot::primary_window()`. Streaming execution uses thread-per-child + mpsc. Implementation order: Feature 1 (log dir) → Feature 3 (--clean) → Feature 2 (screenshots) → Feature 4 (UiScale + tiling) → Feature 5 (streaming pool).
