# Code Standards

- **Pragmatic Rust + Bevy**: Follow rustfmt and clippy. Use Bevy conventions where they matter (system fn signatures, component derives, required components). Don't be dogmatic.
- **No magic numbers**: ALL tunable values (physics constants, timing windows, sizes, speeds) MUST be loaded from RON data. No raw numeric literals in game logic, except in Default impl blocks, and only where a RON will override them if succesfully loaded.
- **No over-engineering**: No abstractions, generics, or indirection until there's a concrete second use case. YAGNI. Three similar lines > premature abstraction.
- **Commits and branches**: See `.claude/rules/commit-format.md` for commit format, `.claude/rules/git.md` for branch naming.

---

## Prelude — Cross-Domain Imports

The `crate::prelude` module provides stable import points for types used across domain boundaries. Use it to avoid verbose, fragile `crate::domain::submodule::Type` imports.

### Usage

- **`use crate::prelude::*`** — curated glob of the most universally used cross-domain types: entity markers (Bolt, Breaker, Cell, Wall), all states, all cross-domain messages, effect containers and active-effect components, and common resources (GameRng, InputActions, PlayfieldConfig).

The prelude submodules (`components`, `messages`, `resources`, `states`) currently export the same set as the curated glob. As more cross-domain types are added, narrower types may be placed in submodules only (not the glob) for files that need them without pulling in everything.

### When to Use

- **Use the prelude** when a file imports 3+ types from 2+ different domains. Replace the verbose cross-domain imports with `use crate::prelude::*` and keep domain-internal imports explicit.
- **Don't use the prelude** for 1-2 cross-domain imports — explicit paths are clearer for small import sets.
- **Don't use the prelude for same-domain imports** — even if a type is in the prelude, import it from your own domain's module path when you're within that domain.

### What Belongs in the Prelude

A type belongs in `crate::prelude` if it is used by **2+ domains** — add it to the appropriate submodule file (`components.rs`, `messages.rs`, `resources.rs`, or `states.rs`). Add it to the curated glob in `prelude/mod.rs` only if it is used by **3+ domains**. Only add re-exports for types that have active consumers through the prelude — unused re-exports cause clippy warnings.

When adding new cross-domain types (components, messages, resources, states), add them to the prelude as consumers are migrated to use it.

### What Does NOT Belong in the Prelude

- Domain-internal types (only used within one domain)
- Plugins, system sets (wiring code only)
- Constants (stay in `crate::shared`)
- Effect dispatch enums (`EffectKind`, `Target`, `Trigger`, `TriggerContext`) — these are well-served by `use crate::effect::*` within the effect domain

### Import Style

- Prefer `crate::` absolute paths over `super::super::` chains (2+ levels). Single `super::` is fine and idiomatic.
- Group related imports from the same module using Rust grouped import syntax.
- In test files, `use crate::` paths are preferred over deep `super::` chains for readability after file splits.

---

## Error Handling — Strict Dev, Lenient Release

**Dev builds** (`cfg(debug_assertions)`): panic aggressively on any unexpected state. Catches bugs fast. If a system encounters something that shouldn't happen, it crashes with a clear message. `debug_assert!` for invariants throughout.

**Release builds**: `warn!` for non-critical issues (missing particle effect, unexpected state). Panic only for truly unrecoverable situations (no breaker entity, corrupt save data). Prefer graceful degradation — a missing sound effect shouldn't crash the game.

**Validation at load time**: Registries validate all RON data on boot. Missing references, invalid values, schema mismatches — all caught before the player reaches the menu. This is the primary defense layer.

---

## Testing — TDD

See `.claude/rules/tdd.md` for the full RED → GREEN → REFACTOR cycle definition, agent boundaries, and RED gate.

**No implementation before failing tests. No exceptions.**

### Test Types

- **Unit tests**: Physics calculations, collision math, state machine transitions, timing windows, upgrade stacking, breaker stats
- **Property-based tests**: Edge cases in physics/collision (use `proptest` — dependency present, planned for physics edge cases)
- **Integration tests**: Use `MinimalPlugins` + headless app to test system interactions
- **Do NOT test**: Rendering, visual output, shader correctness — manual playtesting only

Tests live next to the code they test. Small test suites use in-module `#[cfg(test)]` blocks. Larger suites are split into a sibling `tests.rs` or `tests/` directory module per the [System File Split Convention](layout.md#system-file-split-convention) in `layout.md`.

### Scenario Coverage

Every new gameplay mechanic or system must also be evaluated for **scenario runner coverage**. The scenario runner (`breaker-scenario-runner/`) validates gameplay invariants under automated input (chaos, scripted, hybrid) across hundreds of frames.

When implementing a new feature, ask:
1. **Can existing invariants catch regressions?** If so, ensure existing scenarios exercise the new code path (e.g., a new cell type should appear in at least one scenario layout).
2. **Does this feature introduce a new invariant?** Properties that must always hold (e.g., "chip stack count never exceeds max_stacks", "bolt count never exceeds configured max") should become new `InvariantKind` variants checked every frame. **Every new `InvariantKind` must have a self-test scenario** in `scenarios/self_tests/` that intentionally triggers the violation using `debug_setup` or `frame_mutations` and asserts it fires via `allowed_failures`.
3. **Does this feature need a dedicated scenario?** New mechanics that interact with physics, timing, or state machines benefit from chaos-input stress testing that unit tests cannot replicate.

Existing invariant kinds: `BoltInBounds`, `BoltSpeedAccurate`, `BoltCountReasonable`, `BreakerInBounds`, `NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidDashState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`, `OfferingNoDuplicates`, `MaxedChipNeverOffered`, `ChipStacksConsistent`, `RunStatsMonotonic`, `ChipOfferExpected`, `SecondWindWallAtMostOne`, `ShieldWallAtMostOne`, `PulseRingAccumulation`, `ChainArcCountReasonable`, `AabbMatchesEntityDimensions`, `GravityWellCountReasonable`, `BreakerCountReasonable`.

Scenarios live in `breaker-scenario-runner/scenarios/` organized by category (`mechanic/`, `stress/`, `self_tests/`, `chaos/`).

The coverage manifest (`breaker-scenario-runner/src/coverage.rs`) runs with `--all` (printed before the run) and standalone with `--coverage`. Reports missing self-tests and unused layouts; prints nothing when coverage is complete. All 22 invariants have self-test coverage.

---

## Entity Cleanup — Marker Components

Entities are tagged with cleanup markers that indicate their lifecycle scope. `OnExit` systems query for markers and despawn.

```rust
// Generic cleanup system — one instance per marker type
pub fn cleanup_entities<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// Registered for each state exit that needs cleanup
app.add_systems(OnExit(GameState::Playing), cleanup_entities::<PlayingCleanup>);
app.add_systems(OnExit(GameState::MainMenu), cleanup_entities::<MainMenuCleanup>);
```

Explicit, predictable, easy to debug. Every spawned entity gets a cleanup marker — no entity leaks.

---

## Asset Loading — Preload at Boot

All assets (RON data, textures, audio) are loaded during a single loading screen at startup and kept in memory for the session. No mid-game loading, no hitches between nodes.

For a 2D game of this scope, total asset size is small. Simplicity wins.

**Current boot sequence:**
1. Load RON defaults (playfield, bolt, breaker, cells, input, mainmenu, chipselect, archetype definitions, cell type definitions, node layouts, chip definitions, difficulty curve) via `bevy_asset_loader`
2. Seed config resources from loaded assets — individual per-config seed systems (`seed_bolt_config`, `seed_breaker_config`, `seed_cell_config`, `seed_playfield_config`, `seed_input_config`, `seed_main_menu_config`, `seed_timer_ui_config`, `seed_chip_select_config`, `seed_archetype_registry`, `seed_cell_type_registry`, `seed_node_layout_registry`, `seed_chip_registry`, `seed_difficulty_curve`)
3. Transition to `MainMenu` state

Future phases will add: textures, sprite atlases, audio clips, and cross-reference validation.

---

## Scenario Runner — breaker-scenario-runner

Automated gameplay testing tool in `breaker-scenario-runner/`. A separate workspace crate that is never shipped in release builds.

- **CLI**: `cargo scenario -- -s <name> --visual` (visual debug), `cargo scenario -- --all` (CI/validation), `cargo scenario -- --coverage` (coverage report only)
- **Scenario files**: RON-defined runs (`breaker`, `layout`, `input`, `max_frames`, `disallowed_failures`, `allowed_failures`, etc.) stored in `breaker-scenario-runner/scenarios/`
- **Input strategies**: `Chaos` (seeded random), `Scripted` (deterministic frame-action pairs), `Hybrid` (scripted then chaos)
- **Invariants checked each frame**: `BoltInBounds`, `BoltSpeedAccurate`, `BoltCountReasonable`, `BreakerInBounds`, `NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidDashState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`, `OfferingNoDuplicates`, `MaxedChipNeverOffered`, `ChipStacksConsistent`, `RunStatsMonotonic`, `ChipOfferExpected`, `SecondWindWallAtMostOne`, `ShieldWallAtMostOne`, `PulseRingAccumulation`, `ChainArcCountReasonable`, `AabbMatchesEntityDimensions`, `GravityWellCountReasonable`, `BreakerCountReasonable`
- **Conditional checking**: all invariant checkers are gated on `ScenarioStats::entered_playing` to prevent false positives during initial loading states
- **Log capture**: custom `tracing::Layer` fails the scenario on any WARN/ERROR from `breaker` targets
- **Self-test scenarios**: scenarios in `scenarios/self_tests/` use `allowed_failures` to verify the invariant checker itself fires the expected violation
- **Fail-fast mode**: `--fail-fast` stops the scenario on the first invariant violation; `--no-fail-fast` runs to completion; default is on for `--all`, off for `-s`
- **Run log**: each run writes an async log to `/tmp/breaker-scenario-runner/<YYYY-MM-DD>/<N>.log` via `RunLog` (background mpsc + file IO thread)
- **Output directory**: structured output at `<BASE_DIR>/<YYYY-MM-DD>/<N>/`; run number auto-increments; `--clean` removes all output
- **Window tiling**: parallel visual-mode runs (`--visual --all`) tile game windows in a grid across the screen using `tiling::TilePosition` and env vars (`SCENARIO_WINDOW_X/Y/W/H`)
- **Screenshot-on-violation**: in visual mode with a run log, the runner captures one screenshot per `InvariantKind` on first violation via `ScreenshotTracker` + `capture_violation_screenshots`

The scenario runner uses `ScenarioLayoutOverride` (a resource in `run/node/resources.rs`) to bypass the run-setup screen and inject the specified layout and archetype directly.

CI runs all scenarios headless on Linux via `.github/workflows/ci.yml` (`scenarios` job).

---

## Debug Console — bevy_egui

In-game debug panel built on `bevy_egui` with:

- **Overlay toggles**: hitboxes, velocity vectors, FPS counter
- **Telemetry windows**: bolt info (position, velocity, speed), breaker state (state machine, tilt, velocity, bump state), input actions
- **Bump result tracking**: last bump grade and timing (dev-only FixedUpdate system)
- **Game state label**: current GameState displayed in overlay

Added in Phase 0. The debug console is a development tool, not a player feature — gated behind `#[cfg(feature = "dev")]` in `DebugPlugin::build()`.

Future: live value tweaking, registry browser (when upgrades exist).
