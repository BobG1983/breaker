# Code Standards

- **Pragmatic Rust + Bevy**: Follow rustfmt and clippy. Use Bevy conventions where they matter (system fn signatures, component derives, required components). Don't be dogmatic.
- **No magic numbers**: ALL tunable values (physics constants, timing windows, sizes, speeds) MUST be loaded from RON data. No raw numeric literals in game logic, except in Default impl blocks, and only where a RON will override them if succesfully loaded.
- **No over-engineering**: No abstractions, generics, or indirection until there's a concrete second use case. YAGNI. Three similar lines > premature abstraction.
- **Commits and branches**: See `.claude/rules/commit-format.md` for commit format, `.claude/rules/git.md` for branch naming.

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

Tests live next to the code they test (in-module `#[cfg(test)]` blocks).

### Scenario Coverage

Every new gameplay mechanic or system must also be evaluated for **scenario runner coverage**. The scenario runner (`breaker-scenario-runner/`) validates gameplay invariants under automated input (chaos, scripted, hybrid) across hundreds of frames.

When implementing a new feature, ask:
1. **Can existing invariants catch regressions?** If so, ensure existing scenarios exercise the new code path (e.g., a new cell type should appear in at least one scenario layout).
2. **Does this feature introduce a new invariant?** Properties that must always hold (e.g., "chip stack count never exceeds max_stacks", "bolt count never exceeds configured max") should become new `InvariantKind` variants checked every frame.
3. **Does this feature need a dedicated scenario?** New mechanics that interact with physics, timing, or state machines benefit from chaos-input stress testing that unit tests cannot replicate.

Existing invariant kinds: `BoltInBounds`, `BoltSpeedInRange`, `BoltCountReasonable`, `BreakerInBounds`, `NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidStateTransitions`, `ValidBreakerState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`, `PhysicsFrozenDuringPause`.

Scenarios live in `breaker-scenario-runner/scenarios/` organized by category (`mechanic/`, `stress/`, `self_tests/`).

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

- **CLI**: `cargo scenario -- -s <name> --visual` (visual debug), `cargo scenario -- --all` (CI/validation)
- **Scenario files**: RON-defined runs (`breaker`, `layout`, `input`, `max_frames`, `invariants`) stored in `breaker-scenario-runner/scenarios/`
- **Input strategies**: `Chaos` (seeded random), `Scripted` (deterministic frame-action pairs), `Hybrid` (scripted then chaos)
- **Invariants checked each frame**: `BoltInBounds`, `BoltSpeedInRange`, `BoltCountReasonable`, `BreakerInBounds`, `NoEntityLeaks`, `NoNaN`, `TimerNonNegative`, `ValidStateTransitions`, `ValidBreakerState`, `TimerMonotonicallyDecreasing`, `BreakerPositionClamped`, `PhysicsFrozenDuringPause`
- **Log capture**: custom `tracing::Layer` fails the scenario on any WARN/ERROR from `breaker` targets
- **Self-test scenarios**: scenarios in `scenarios/self_tests/` use `expected_violations` to verify the invariant checker itself

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
