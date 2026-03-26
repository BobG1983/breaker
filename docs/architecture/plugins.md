# Plugin Architecture

## Workspace Layout

Cargo workspace with peer crates at the repository root. Directory names follow `breaker-<name>` convention.

```
brickbreaker/                 # Repository root (workspace)
├── Cargo.toml                # Workspace manifest — members, shared lints, profiles
├── .cargo/config.toml        # Dev aliases (cargo dev, cargo dtest, cargo dscenario, etc.)
├── breaker-game/             # Main game crate (binary + library)
│   ├── Cargo.toml            # Package: breaker (lib) / brickbreaker (bin)
│   ├── src/                  # Game source (see Domain Layout below)
│   └── assets/               # RON data files, shaders, textures, audio
├── rantzsoft_spatial2d/      # Game-agnostic 2D spatial plugin (Position2D, propagation, interpolation)
│   ├── Cargo.toml            # Package: rantzsoft_spatial2d
│   └── src/                  # Components, systems, DrawLayer trait, propagation enums
├── rantzsoft_physics2d/      # Game-agnostic 2D physics primitives (quadtree, CCD, CollisionLayers, DistanceConstraint)
│   ├── Cargo.toml            # Package: rantzsoft_physics2d
│   └── src/                  # Aabb2D, CollisionLayers, DistanceConstraint, quadtree, CCD
├── rantzsoft_defaults/       # Re-exports the GameConfig derive macro
│   ├── Cargo.toml
│   └── src/lib.rs
├── rantzsoft_defaults_derive/ # Proc-macro crate: #[derive(GameConfig)] for RON defaults loading
│   ├── Cargo.toml
│   └── src/lib.rs
├── breaker-scenario-runner/  # Automated gameplay testing tool (dev-only binary)
│   ├── Cargo.toml            # Package: breaker_scenario_runner
│   ├── src/                  # Runner source (types, lifecycle, invariants, input, log_capture)
│   └── scenarios/            # RON scenario files (crate-local, never shipped)
└── docs/                     # Design docs, architecture, build plan
```

**Naming convention:** Game-specific crate directories use `breaker-<name>`; game-agnostic reusable crates use `rantzsoft_<name>` (see `.claude/rules/rantzsoft-crates.md`). Cargo package names use underscores (`brickbreaker`, `breaker_scenario_runner`, `rantzsoft_spatial2d`, etc.). `rantzsoft_*` crates contain zero game-specific code and may be extracted to separate repos when reuse is needed.

## Domain Layout

One Bevy plugin per **game domain** (breaker, bolt, cells, etc.) — not one per Bevy system function. Each domain plugin encapsulates all the Bevy systems, components, resources, and messages related to that domain.

Inside `breaker-game/`:

```
src/
├── lib.rs            # Library root: declares all domain modules
├── main.rs           # Binary entry point: calls lib to build and run
├── app.rs            # App — constructs the Bevy App with DefaultPlugins + Game
├── game.rs           # Game — PluginGroup that wires together all domain plugins
├── shared/           # Passive types: GameState, PlayingState, cleanup markers, shared math
├── screen/           # Screen state registration, transitions, cleanup systems
├── input/            # Raw keyboard input to GameAction translation
├── breaker/          # Breaker mechanics, state machine, bump system
├── effect/           # Effect system — data-driven EffectNode trigger/effect evaluation and dispatch (top-level domain)
├── bolt/             # Bolt physics, reflection model, speed management, CCD collision detection, chain bolts
├── cells/            # Cell types, grid layout, destruction
├── wall/             # Invisible boundary entities (left, right, ceiling)
├── chips/            # Chip system — template loading, EffectNode effects, observer-based application; EvolutionRegistry for evolution recipes
├── fx/               # Cross-cutting visual effects (fade-out, node transition overlays)
├── run/              # Run state, node sequencing (node/ sub-domain), timer, RunStats accumulation, HighlightTracker, highlight detection (highlights/ sub-domain), spawn_highlight_text juice
├── audio/            # Event-driven audio, adaptive intensity (stub — Phase 6)
├── ui/               # HUD, menus, chip selection screen
└── debug/            # Dev tooling: overlays, telemetry, hot-reload, recording (sub-domains)
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**Scenario runner exception** — `bolt`, `breaker`, `chips`, `effect`, `input`, and `run` are declared as `pub mod` in `lib.rs` (not `pub(crate)`) because `breaker-scenario-runner` needs cross-crate access to their components, resources, and system sets for entity tagging, input injection, invariant checking, and ordering constraints. This mirrors the existing debug domain exception.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins. Plugin registration order: `InputPlugin`, `ScreenPlugin`, `RantzSpatial2dPlugin::<GameDrawLayer>`, `RantzPhysics2dPlugin`, `WallPlugin`, `BreakerPlugin`, `EffectPlugin`, `BoltPlugin`, `CellsPlugin`, `ChipsPlugin`, `FxPlugin`, `RunPlugin`, `AudioPlugin`, `UiPlugin`, `DebugPlugin`.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- **Writes** to other domains only through messages — no direct mutation of another domain's components or resources
- **Reads** from other domains' types (components, messages, resources) are normal ECS patterns and not violations — see "Cross-Domain Read Access" below

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `EffectSystems` (`effect/sets.rs`), `UiSystems` (`ui/sets.rs`), `NodeSystems` (`run/node/sets.rs`). The external `rantzsoft_physics2d::plugin::PhysicsSystems` set (`MaintainQuadtree`, `EnforceDistanceConstraints`) is also used for ordering against the quadtree maintenance system. See [ordering.md](ordering.md) for the full table and usage rules.

## Cross-Domain Read Access

The architectural boundary is about **writes** (mutations), not reads. Domains freely **read** other domains' types — components, message types, resources — via standard ECS queries. This is normal Bevy and not a violation:

- **bolt** (collision systems) reads `Piercing`, `PiercingRemaining`, `DamageBoost` (chips domain) from bolt entities, `CellHealth`, `CellWidth`, `CellHeight` (cells domain) from cell entities, and `BreakerWidth`, `BreakerHeight` (breaker domain) from the breaker entity. The bolt collision systems also write message types owned by other domains (e.g., writing a cells-domain `DamageCell` message). This is expected — collision is a cross-cutting concern now hosted in the bolt domain.
- **cells** reads `DamageBoost` (chips domain) from bolt entities in `handle_cell_hit`.
- **breaker** reads `TiltControlBoost`, `BreakerSpeedBoost`, `BumpForceBoost` (chips domain) from its own entity.
- **effect** reads `BumpPerformed`, `BumpWhiffed` (breaker domain), `BoltHitCell`, `BoltHitBreaker`, `BoltHitWall`, `BoltLost` (bolt domain), and `RequestCellDestroyed` / `CellDestroyedAt` (cells domain) messages in bridge systems.

**The rule**: any domain may `use crate::other_domain::*` for read-only queries and message consumption. No domain writes to another domain's canonical components or resources directly — that flows through messages. The `debug/` domain is the sole exception (read AND write, compiled out of release builds).

## Debug Domain — Cross-Domain Exception

The `debug/` domain (gated behind `#[cfg(feature = "dev")]`) is the **only domain permitted to read AND write other domains' resources and components** directly. This is an accepted architectural exception because:

- Hot-reload systems must write to `Res<*Config>` and insert/update entity components across all domains
- Telemetry systems must read components and resources from every domain for display
- Recording systems capture `GameAction` inputs for later scripted playback
- All debug code is compiled out of release builds — it cannot introduce production coupling

This exception does **not** extend to other domains. Production code still communicates through messages only.

## Effect Domain — Self-Registration Pattern

The `effect/` domain uses a self-registration pattern where each leaf effect is fully self-contained in a single file and registers itself with the app.

### Target Structure

```
effect/
  effect_nodes/        # EffectNode tree logic
    until.rs           # UntilTimers, UntilTriggers, tick_until_timers, check_until_triggers
  effects/             # Leaf effect handlers (one file per effect)
    speed_boost.rs     # SpeedBoostFired, handle_speed_boost, register()
    damage_boost.rs    # DamageBoostApplied, handle_damage_boost, register()
    shockwave.rs       # ShockwaveFired, handle_shockwave, register()
    life_lost.rs       # LoseLifeFired, handle_life_lost, spawn_lives_display, register()
    chain_bolt.rs      # ChainBoltFired, handle_chain_bolt, register()
    ramping_damage.rs  # RampingDamageApplied, handle_ramping_damage, increment/reset systems, register()
    ... (one file per effect — ~20 total)
  triggers/            # Bridge systems (one file per trigger event)
    on_impact.rs       # bridge_cell_impact, bridge_wall_impact, bridge_breaker_impact
    on_bump.rs         # bridge_bump, bridge_bump_whiff (all bump grades including NoBump)
    on_no_bump.rs      # bridge_no_bump
    on_bolt_lost.rs    # bridge_bolt_lost
    on_death.rs        # bridge_cell_death, bridge_bolt_death, cleanup systems, apply_once_nodes
    on_timer.rs        # bridge_timer_threshold
  helpers.rs           # Shared bridge helpers (evaluate_entity_chains, evaluate_active_chains, arm_bolt, etc.)
  active.rs            # ActiveEffects resource (Vec<(Option<String>, EffectNode)>)
  armed.rs             # ArmedEffects component (Vec<(Option<String>, EffectNode)> on bolt entities)
  definition.rs        # EffectNode, Trigger, Effect, EffectTarget, EffectChains, Target, ImpactTarget enums
  evaluate.rs          # evaluate_node — pure trigger matching, NodeEvalResult
  typed_events.rs      # Re-exports all typed events; fire_typed_event / fire_passive_event dispatch helpers
  registry.rs          # BreakerRegistry re-export (canonical: breaker/registry.rs)
  sets.rs              # EffectSystems::Bridge set
  plugin.rs            # EffectPlugin — calls each effect's register() and wires trigger bridges
```

### Effect File Pattern

Each leaf effect file in `effects/` owns its complete lifecycle:

```rust
// effect/effects/speed_boost.rs

// Typed event (what gets fired by bridges)
pub(crate) struct SpeedBoostFired { ... }

// Active state (vec of applied multipliers on the entity)
pub(crate) struct ActiveSpeedBoosts(pub Vec<f32>);

// Removal message (fired by Until on expiry)
pub(crate) struct RemoveSpeedBoost { pub entity: Entity, pub multiplier: f32 }

// Apply observer (pushes to vec when typed event fires)
fn handle_speed_boost(trigger: On<SpeedBoostFired>, ...) { ... }

// Recalculation system (recalculates stat from vec every tick)
fn apply_speed_boosts(query: Query<(&mut Velocity2D, &BoltBaseSpeed, &ActiveSpeedBoosts), ...>) { ... }

// Removal observer (removes entry from vec, end-of-frame)
fn handle_remove_speed_boost(trigger: On<RemoveSpeedBoost>, ...) { ... }

// Self-registration
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_speed_boost)
       .add_message::<RemoveSpeedBoost>()
       .add_observer(handle_remove_speed_boost)
       .add_systems(FixedUpdate, apply_speed_boosts.in_set(EffectSystems::Apply));
}
```

`EffectPlugin::build()` calls each effect's `register()`:

```rust
fn build(&self, app: &mut App) {
    speed_boost::register(app);
    damage_boost::register(app);
    shockwave::register(app);
    piercing::register(app);
    // ...
}
```

### Passive Effects and Until Integration

Passive effects (SpeedBoost, DamageBoost, Piercing, SizeBoost, BumpForce) track state in per-entity vecs. The actual stat is **recalculated from the vec** — no incremental mutation.

**Until removal**: `Until` nodes have zero knowledge of effect internals. On expiry, they fire `Remove*` messages for each passive child they applied. Each effect's own removal observer handles cleanup. The recalculation system picks up the change next tick.

**Non-passive children in Until**: `When`/`Once` nodes nested inside `Until` live in the `UntilTimers`/`UntilTriggers` container. Bridges evaluate them while the Until is alive. When the Until expires, the container is removed — armed triggers are gone. No removal message needed.

### Chain Ownership Model

Three effect stores serve different roles:

- **`ActiveEffects`** — global resource (`Vec<(Option<String>, EffectNode)>`). Populated by `init_breaker` from the breaker definition and by `dispatch_chip_effects` when triggered chip chains are selected. Bridge helpers sweep this for global triggers (BoltLost, BumpWhiff, NoBump, CellDestroyed).
- **`ArmedEffects`** — component on bolt entities. Partially resolved `When` trees waiting for a deeper trigger. Consumed on Fire, replaced on re-Arm.
- **`EffectChains`** — component on individual entities (bolts, cells). Entity-local chains evaluated by `evaluate_entity_chains`. Used for `Once`-wrapped one-shot effects and cell-specific chains.

**Chip dispatch**: `dispatch_chip_effects` fires passive events (for `OnSelected` leaves) immediately via `fire_passive_event` and pushes all other triggered chains into `ActiveEffects`.

**Evaluation routing**: Bridge systems call `evaluate_active_chains` (global triggers: BoltLost, BumpWhiff, NoBump) or `evaluate_entity_chains` + `evaluate_armed` (entity-specific triggers: Impact, Bump, Death). `evaluate_node` peels one layer, returning `Fire(effect)`, `Arm(remaining_chain)`, or `NoMatch`.

**Note**: `CellDestroyed` trigger evaluation is handled by `bridge_cell_death` in `on_death.rs` — reads `RequestCellDestroyed` while the cell is still alive, evaluates chains, then `cleanup_destroyed_cells` despawns the entity.

**Arm routing**: When `evaluate_node` returns `Arm(remaining)`, the remaining chain is pushed to the bolt entity's `ArmedEffects` component. Subsequent trigger events re-evaluate armed chains via `evaluate_armed`.
