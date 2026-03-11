# Architecture

Technical decisions for how the game is built. See `DESIGN.md` for *why* (game feel), `PLAN.md` for *when* (build phases), `TERMINOLOGY.md` for vocabulary.

---

## Engine & Stack

- **Bevy 0.18.1** — 2D only (`default-features = false, features = ["2d"]`)
- **Custom physics** — No rapier. Breakout physics are specialized (angle overwrite, no perfect verticals). Full control needed.
- **Data format** — Hybrid: type-safe mechanics in Rust, tweakable content in RON files
- **Debug UI** — `bevy_egui` for in-game debug console (added Phase 0)

---

## Plugin Architecture

One Bevy plugin per **game domain** (breaker, bolt, cells, etc.) — not one per Bevy system function. Each domain plugin encapsulates all the Bevy systems, components, resources, and messages related to that domain.

Everything lives inside a single `brickbreaker` crate:

```
src/
├── lib.rs            # Library root: declares all domain modules
├── main.rs           # Binary entry point: calls lib to build and run
├── app.rs            # App — constructs the Bevy App with DefaultPlugins + Game
├── game.rs           # Game — PluginGroup that wires together all domain plugins
├── shared.rs         # Passive types: GameState, PlayingState, cleanup markers, constants
├── screen/           # Screen state registration, transitions, cleanup systems
├── breaker/          # Breaker mechanics, state machine, bump system
├── bolt/             # Bolt physics, reflection model, speed management
├── cells/            # Cell types, grid layout, destruction
├── upgrades/         # Amps, Augments, Overclocks system
├── run/              # Run state, seeded node sequencing, timer, difficulty scaling
├── physics/          # Quadtree, collision detection, collision response
├── audio/            # Event-driven audio, adaptive intensity
├── ui/               # HUD, menus, upgrade selection screen
├── debug/            # bevy_egui debug console, overlays
└── assets/           # RON data files, shaders, textures, audio
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- Communicates outward only through messages — no direct cross-module imports for data flow

### Domain Folder Layout

Every domain folder follows this canonical internal structure:

```
src/<domain>/
├── mod.rs           # Re-exports ONLY — pub mod declarations, pub use re-exports. No logic, no types.
├── plugin.rs        # The Plugin impl. Registers systems, messages, states. One per domain.
├── components.rs    # All #[derive(Component)] types for this domain.
├── messages.rs      # All #[derive(Message)] types for this domain.
├── resources.rs     # All #[derive(Resource)] types for this domain.
└── systems/
    ├── mod.rs       # Re-exports ONLY — pub mod + pub use for each system.
    └── <name>.rs    # One file per system function (or tightly related group).
```

**Rules:**
- **`mod.rs`** is a routing file. It contains `pub mod` and `pub use` statements only. No `fn`, `struct`, `enum`, or `impl`.
- **`plugin.rs`** is the only file that wires things to the Bevy `App` — system registration, message registration, state registration all happen here.
- **`components.rs`**, **`messages.rs`**, **`resources.rs`** — one file each per category. Omit the file if the domain has none of that category (e.g., no `messages.rs` if the domain sends no messages).
- **`systems/`** — one `.rs` file per system function, or per tightly-coupled group (e.g., a system + its helper). Files are named after the system. `systems/mod.rs` only re-exports.
- No `utils.rs`, `helpers.rs`, `common.rs`, or `types.rs`. If it doesn't fit the categories above, it probably belongs in an existing file or a different domain.

---

## Messages — Inter-System Communication

Systems are decoupled through Bevy 0.18 messages (`#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`). The breaker plugin doesn't import audio. The cell plugin doesn't import upgrades. Messages connect them.

**Example message types:**

| Message | Sent By | Consumed By |
|---------|---------|-------------|
| `BoltHitBreaker` | physics | audio, upgrades, UI |
| `BoltHitCell` | physics | upgrades, cells, audio |
| `CellDestroyed` | cells | run (progress tracking), upgrades (overclock triggers), audio |
| `BoltLost` | physics | breaker (applies penalty per breaker trait) |
| `NodeCleared` | run | state machine, UI |
| `UpgradeSelected` | UI | upgrades (apply effects) |
| `BumpPerformed { grade }` | breaker | audio, upgrades (overclock triggers) |
| `TimerExpired` | run | state machine |

---

## State Management

Bevy `States` for top-level game state. `SubStates` where a state only exists within a parent.

**Top-level states (`GameState`):**
- `Loading` — asset preload (default/initial state)
- `MainMenu`
- `RunSetup` — breaker/seed selection
- `Playing` — active node (see sub-states below)
- `UpgradeSelect` — timed upgrade selection
- `RunEnd` — win/lose screen
- `MetaProgression` — between-run Flux spending

**Playing sub-states (`PlayingState`):**
- `Active` — normal gameplay (default when entering `Playing`)
- `Paused` — all gameplay frozen

`PlayingState` only exists when `GameState::Playing` is active — it is automatically created and destroyed by Bevy's sub-state lifecycle. Pausing is modeled as a sub-state (not top-level) because you can only pause from active gameplay. This constraint is encoded in the type system.

Systems that should freeze during pause use `run_if(in_state(PlayingState::Active))`. Systems that should run regardless of pause (e.g., pause menu UI) use `run_if(in_state(GameState::Playing))`.

**Passive types vs. active logic:** `GameState`, `PlayingState`, cleanup markers, and playfield constants are passive types defined in `shared.rs` (imported by all domains). State registration, transitions, and cleanup systems live in the `screen/` domain plugin.

---

## Physics — FixedUpdate + Quadtree

**Timestep:** All physics runs in `FixedUpdate` for deterministic behavior. This is required for seeded run reproducibility — the same seed must produce identical physics across hardware. Visual interpolation smooths rendering between fixed ticks.

**Collision — Quadtree:**
- Persistent quadtree `Resource` that entities insert into on spawn, update on move, and remove from on despawn
- Handles both static cell grids and moving cells (active nodes, Phase 6+)
- Bolt-vs-cell, bolt-vs-breaker, bolt-vs-wall queries through the quadtree

**Bolt reflection:**
- Direction entirely overwritten on breaker contact — no incoming angle carryover
- Reflection angle determined by: hit position on breaker, breaker tilt state, bump grade
- No perfectly vertical or horizontal reflections — always enforce minimum angle

---

## System Ordering — Loose with Key Constraints

No named phase sets or global pipeline. Only add `.before()` / `.after()` where actual data dependencies exist. Let Bevy parallelize everything else.

```rust
app.add_systems(FixedUpdate, (
    // Ordered chain — data flows between these
    apply_input.before(move_breaker),
    move_breaker.before(move_bolt),
    move_bolt.before(detect_collisions),
    detect_collisions.before(apply_collision_response),

    // No ordering constraint — run in parallel
    update_node_timer,
    update_quadtree,
    check_bolt_lost,

    // Only needs to run after what it reacts to
    play_hit_sounds.after(detect_collisions),
).run_if(in_state(GameState::Playing)));
```

Ordering is added when systems have proven data dependencies, not speculatively. If a system doesn't read another system's output, it runs freely.

---

## Content Identity — Enum Behaviors + RON Instances

**Behaviors** are Rust enums. **Content instances** are RON files that compose and tune those behaviors.

```rust
// Behavior types — exhaustive, matchable, compiler-checked
#[derive(Debug, Clone, Deserialize)]
pub enum AmpEffect {
    Piercing(u32),
    DamageBoost(f32),
    SpeedBoost(f32),
    Ricochet(u32),
    SizeBoost(f32),
}

// Content instance — data-driven, no recompile to add
#[derive(Debug, Deserialize)]
pub struct AmpDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub effects: Vec<AmpEffect>,  // Multiple effects per upgrade
    pub rarity: Rarity,
    pub stacks: bool,
}
```

```ron
// assets/amps/voltspike.ron
/* @[brickbreaker::upgrades::AmpDefinition] */
(
    id: "voltspike",
    name: "Voltspike",
    description: "Pierces through cells and hits harder",
    effects: [
        Piercing(1),
        DamageBoost(2.0),
    ],
    rarity: Uncommon,
    stacks: true,
)
```

**Adding new content:** new RON file, no recompile. **Adding new behavior types:** new enum variant, requires recompile (appropriate — new behavior means new code).

Registries (`AmpRegistry`, `AugmentRegistry`, `OverclockRegistry`) are `Resource`s that load and validate all RON definitions at boot. Game logic looks up definitions through the registry, never matches on raw ID strings.

### RON Validation — ron-lsp

Every RON file MUST include a type annotation comment on the first line linking it to the Rust type it deserializes into:

```ron
// assets/amps/voltspike.ron
/* @[brickbreaker::upgrades::AmpDefinition] */
(
    id: "voltspike",
    ...
)
```

[`ron-lsp`](https://github.com/jasonjmcghee/ron-lsp) uses these annotations to validate RON files against actual Rust struct/enum definitions — catching type mismatches, missing fields, and invalid enum variants without running the game. Run `ron-lsp check .` to validate all annotated RON files in bulk.

---

## Upgrade Application — Components on Entities

When a player selects an upgrade, it becomes a **component on the bolt or breaker entity**. Systems query for specific upgrade components to apply their effects.

```rust
// Active upgrade component on the bolt entity
#[derive(Component)]
pub struct ActiveAmp {
    pub definition: AmpDefinition,
}

// Systems query for active upgrades
fn apply_piercing(
    bolts: Query<(&Bolt, &ActiveAmp)>,
) {
    for (bolt, amp) in &bolts {
        for effect in &amp.definition.effects {
            match effect {
                AmpEffect::Piercing(count) => { /* ... */ }
                _ => {}
            }
        }
    }
}
```

This means upgrades can carry state (remaining pierce count, cooldown timers) and multiple upgrades of the same type stack naturally as multiple components.

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
1. Load all RON definitions → build registries (Amp, Augment, Overclock, Breaker, Cell)
2. Load all textures and sprite atlases
3. Load all audio clips
4. Validate cross-references (RON files referencing each other)
5. Transition to `MainMenu` state

---

## Error Handling — Strict Dev, Lenient Release

**Dev builds** (`cfg(debug_assertions)`): panic aggressively on any unexpected state. Catches bugs fast. If a system encounters something that shouldn't happen, it crashes with a clear message. `debug_assert!` for invariants throughout.

**Release builds**: `warn!` for non-critical issues (missing particle effect, unexpected state). Panic only for truly unrecoverable situations (no breaker entity, corrupt save data). Prefer graceful degradation — a missing sound effect shouldn't crash the game.

**Validation at load time**: Registries validate all RON data on boot. Missing references, invalid values, schema mismatches — all caught before the player reaches the menu. This is the primary defense layer.

---

## Debug Console — bevy_egui

In-game debug panel built on `bevy_egui` with:

- **Overlay toggles**: hitboxes, velocity vectors, quadtree visualization, state labels, FPS
- **Live value tweaking**: physics constants, timing windows, speed values — immediate feedback without recompile
- **State inspection**: current game state, active upgrades, breaker state machine, bolt velocity
- **Registry browser**: view loaded Amp/Augment/Overclock definitions

Added in Phase 0. The debug console is a development tool, not a player feature — compiled out of release builds or hidden behind a flag.

---

## Code Standards

- **Pragmatic Rust + Bevy**: Follow rustfmt and clippy. Use Bevy conventions where they matter (system fn signatures, component derives, required components). Don't be dogmatic.
- **No magic numbers**: ALL tunable values (physics constants, timing windows, sizes, speeds) must be named constants or loaded from RON data. No raw numeric literals in game logic.
- **No over-engineering**: No abstractions, generics, or indirection until there's a concrete second use case. YAGNI. Three similar lines > premature abstraction.
- **Conventional Commits**: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:` with optional scope. Branch naming: `feature/*`, `fix/*`, `refactor/*`.

---

## Testing — TDD

Write tests FIRST for all game logic:

- **Unit tests**: Physics calculations, collision math, state machine transitions, timing windows, upgrade stacking, breaker stats
- **Property-based tests**: Edge cases in physics/collision (use `proptest` or `quickcheck`)
- **Integration tests**: Use `MinimalPlugins` + headless app to test system interactions
- **Do NOT test**: Rendering, visual output, shader correctness — manual playtesting only

Tests live next to the code they test (in-module `#[cfg(test)]` blocks).
