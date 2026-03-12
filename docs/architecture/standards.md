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

## Testing — TDD

Write tests FIRST for all game logic:

- **Unit tests**: Physics calculations, collision math, state machine transitions, timing windows, upgrade stacking, breaker stats
- **Property-based tests**: Edge cases in physics/collision (use `proptest` or `quickcheck`)
- **Integration tests**: Use `MinimalPlugins` + headless app to test system interactions
- **Do NOT test**: Rendering, visual output, shader correctness — manual playtesting only

Tests live next to the code they test (in-module `#[cfg(test)]` blocks).

---

## Entity Cleanup — Marker Components

Entities are tagged with cleanup markers that indicate their lifecycle scope. `OnExit` systems query for markers and despawn.

```rust
#[derive(Component)]
pub struct CleanupOnNodeExit;

#[derive(Component)]
pub struct CleanupOnRunEnd;

// Runs when leaving Playing state
fn cleanup_node_entities(
    mut commands: Commands,
    query: Query<Entity, With<CleanupOnNodeExit>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

Explicit, predictable, easy to debug. Every spawned entity gets a cleanup marker — no entity leaks.

---

## Asset Loading — Preload at Boot

All assets (RON data, textures, audio) are loaded during a single loading screen at startup and kept in memory for the session. No mid-game loading, no hitches between nodes.

For a 2D game of this scope, total asset size is small. Simplicity wins.

**Boot sequence:**
1. Load all RON definitions -> build registries (Amp, Augment, Overclock, Breaker, Cell)
2. Load all textures and sprite atlases
3. Load all audio clips
4. Validate cross-references (RON files referencing each other)
5. Transition to `MainMenu` state

---

## Debug Console — bevy_egui

In-game debug panel built on `bevy_egui` with:

- **Overlay toggles**: hitboxes, velocity vectors, quadtree visualization, state labels, FPS
- **Live value tweaking**: physics constants, timing windows, speed values — immediate feedback without recompile
- **State inspection**: current game state, active upgrades, breaker state machine, bolt velocity
- **Registry browser**: view loaded Amp/Augment/Overclock definitions

Added in Phase 0. The debug console is a development tool, not a player feature — compiled out of release builds or hidden behind a flag.
