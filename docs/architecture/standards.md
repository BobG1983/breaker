# Code Standards

- **Pragmatic Rust + Bevy**: Follow rustfmt and clippy. Use Bevy conventions where they matter (system fn signatures, component derives, required components). Don't be dogmatic.
- **No magic numbers**: ALL tunable values (physics constants, timing windows, sizes, speeds) MUST be loaded from RON data. No raw numeric literals in game logic, except in Default impl blocks, and only where a RON will override them if succesfully loaded.
- **No over-engineering**: No abstractions, generics, or indirection until there's a concrete second use case. YAGNI. Three similar lines > premature abstraction.
- **Conventional Commits**: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:` with optional scope. Branch naming: `feature/*`, `fix/*`, `refactor/*`.

---

## Error Handling — Strict Dev, Lenient Release

**Dev builds** (`cfg(debug_assertions)`): panic aggressively on any unexpected state. Catches bugs fast. If a system encounters something that shouldn't happen, it crashes with a clear message. `debug_assert!` for invariants throughout.

**Release builds**: `warn!` for non-critical issues (missing particle effect, unexpected state). Panic only for truly unrecoverable situations (no breaker entity, corrupt save data). Prefer graceful degradation — a missing sound effect shouldn't crash the game.

**Validation at load time**: Registries validate all RON data on boot. Missing references, invalid values, schema mismatches — all caught before the player reaches the menu. This is the primary defense layer.

---

## Testing — TDD (RED → GREEN → REFACTOR)

Write tests FIRST for all game logic. The full cycle is mandatory:

1. **RED** — Write the failing test(s) that describe the desired behavior. Run them — they **must fail** before you write any implementation code. If the test passes immediately, the test is wrong.
2. **GREEN** — Write the minimum code to make the tests pass. No more, no less.
3. **REFACTOR** — Clean up names, eliminate duplication, improve structure. Tests must still pass after refactoring.

**No implementation before failing tests. No exceptions.**

- **Unit tests**: Physics calculations, collision math, state machine transitions, timing windows, upgrade stacking, breaker stats
- **Property-based tests**: Edge cases in physics/collision (use `proptest` — dependency present, planned for physics edge cases)
- **Integration tests**: Use `MinimalPlugins` + headless app to test system interactions
- **Do NOT test**: Rendering, visual output, shader correctness — manual playtesting only

Tests live next to the code they test (in-module `#[cfg(test)]` blocks).

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
1. Load RON defaults (playfield, bolt, breaker, cells, input, mainmenu) via `bevy_asset_loader`
2. Seed config resources from loaded defaults (`seed_configs_from_defaults`)
3. Transition to `MainMenu` state

Future phases will add: upgrade registries (Amp, Augment, Overclock), textures, sprite atlases, audio clips, and cross-reference validation.

---

## Debug Console — bevy_egui

In-game debug panel built on `bevy_egui` with:

- **Overlay toggles**: hitboxes, velocity vectors, FPS counter
- **Telemetry windows**: bolt info (position, velocity, speed), breaker state (state machine, tilt, velocity, bump state), input actions
- **Bump result tracking**: last bump grade and timing (dev-only FixedUpdate system)
- **Game state label**: current GameState displayed in overlay

Added in Phase 0. The debug console is a development tool, not a player feature — gated behind `#[cfg(feature = "dev")]` in `DebugPlugin::build()`.

Future: live value tweaking, registry browser (when upgrades exist).
