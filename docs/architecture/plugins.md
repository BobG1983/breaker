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
├── behaviors/        # Archetype behavior system — unified TriggerChain evaluation and dispatch (top-level domain)
├── bolt/             # Bolt physics, reflection model, speed management, CCD collision detection, chain bolts
├── cells/            # Cell types, grid layout, destruction
├── wall/             # Invisible boundary entities (left, right, ceiling)
├── chips/            # Chip system — template loading, unified TriggerChain effects, observer-based application; EvolutionRegistry for evolution recipes
├── fx/               # Cross-cutting visual effects (fade-out, node transition overlays)
├── run/              # Run state, node sequencing (node/ sub-domain), timer, RunStats accumulation, HighlightTracker, highlight detection (10 systems), spawn_highlight_text juice
├── audio/            # Event-driven audio, adaptive intensity (stub — Phase 6)
├── ui/               # HUD, menus, chip selection screen
└── debug/            # Dev tooling: overlays, telemetry, hot-reload, recording (sub-domains)
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**Scenario runner exception** — `bolt`, `breaker`, `input`, `physics`, and `run` are declared as `pub mod` in `lib.rs` (not `pub(crate)`) because `breaker-scenario-runner` needs cross-crate access to their components, resources, and system sets for entity tagging, input injection, invariant checking, and ordering constraints. This mirrors the existing debug domain exception.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins. It adds `RantzSpatial2dPlugin::<GameDrawLayer>` and `rantzsoft_physics2d::plugin::RantzPhysics2dPlugin` before the game domain plugins, so spatial propagation and quadtree maintenance are available to all domains.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- **Writes** to other domains only through messages — no direct mutation of another domain's components or resources
- **Reads** from other domains' types (components, messages, resources) are normal ECS patterns and not violations — see "Cross-Domain Read Access" below

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `BehaviorSystems` (`behaviors/sets.rs`), `UiSystems` (`ui/sets.rs`), `NodeSystems` (`run/node/sets.rs`). The external `rantzsoft_physics2d::plugin::PhysicsSystems` set (`MaintainQuadtree`, `EnforceDistanceConstraints`) is also used for ordering against the quadtree maintenance system. See [ordering.md](ordering.md) for the full table and usage rules.

## Cross-Domain Read Access

The architectural boundary is about **writes** (mutations), not reads. Domains freely **read** other domains' types — components, message types, resources — via standard ECS queries. This is normal Bevy and not a violation:

- **bolt** (collision systems) reads `Piercing`, `PiercingRemaining`, `DamageBoost` (chips domain) from bolt entities, `CellHealth`, `CellWidth`, `CellHeight` (cells domain) from cell entities, and `BreakerWidth`, `BreakerHeight` (breaker domain) from the breaker entity. The bolt collision systems also write message types owned by other domains (e.g., writing a cells-domain `DamageCell` message). This is expected — collision is a cross-cutting concern now hosted in the bolt domain.
- **cells** reads `DamageBoost` (chips domain) from bolt entities in `handle_cell_hit`.
- **breaker** reads `TiltControlBoost`, `BreakerSpeedBoost`, `BumpForceBoost` (chips domain) from its own entity.
- **behaviors** reads `BumpPerformed`, `BumpWhiffed` (breaker domain), `BoltHitCell`, `BoltHitBreaker`, `BoltHitWall`, `BoltLost` (bolt domain), and `CellDestroyed` (cells domain) messages in bridge systems.

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
    until.rs           # UntilTimers, tick_until_timers, check_until_triggers
    once.rs            # apply_once_nodes, Once consumption
    when.rs            # When evaluation logic
  effects/             # Leaf effect handlers (one file per effect)
    speed_boost.rs     # ActiveSpeedBoosts, apply_speed_boosts, handle_speed_boost, SpeedBoostFired, RemoveSpeedBoost, register()
    damage_boost.rs    # ActiveDamageBoosts, handle_damage_boost, DamageBoostApplied, RemoveDamageBoost, register()
    shockwave.rs       # ShockwaveFired, handle_shockwave, register()
    ...
  triggers/            # Bridge systems (one file per trigger event)
    on_impact.rs       # bridge_cell_impact, bridge_wall_impact, bridge_breaker_impact
    on_bump.rs         # bridge_bump (all bump grades)
    on_perfect_bump.rs # bridge_perfect_bump
    on_bolt_lost.rs    # bridge_bolt_lost
    on_cell_destroyed.rs  # bridge_cell_destroyed
    on_death.rs        # bridge_cell_death, bridge_bolt_death
    on_node_timer_threshold.rs  # bridge_timer_threshold
    on_selected.rs     # dispatch_chip_effects (chip selection)
  helpers.rs           # Shared bridge helpers (evaluate_entity_chains, etc.)
  definition.rs        # EffectNode, Trigger, Effect, EffectTarget, EffectChains, Target enums
  evaluate.rs          # evaluate_node — pure trigger matching
  plugin.rs            # EffectPlugin — calls each effect's register() and wires triggers
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

Chains live on the entity whose events trigger them. Two stores per entity:

- **`EffectChains`** — permanent source of truth. Populated by breaker init, chip dispatch, and `On` node. Never modified by trigger evaluation.
- **`ArmedEffects`** — temporary working set. Partially resolved `When` trees. Consumed on Fire, replaced on re-Arm.

**Chip dispatch**: If outermost node is `On(target, children)`, push children to specified target entity (explicit control). Otherwise push to the entity that owns the outermost trigger: bump triggers → breaker, impact triggers → bolt, death → dying entity, selected → fire immediately.

**Evaluation routing**: Entity-specific triggers (Impact, Bump, Death) evaluate only the relevant entity. Global triggers (CellDestroyed, BoltLost) sweep ALL entities with EffectChains — chains fire wherever they were dispatched to.

**Note**: `CellDestroyed` is a trigger (evaluated by the effect system), not a message. The bridge reads `RequestCellDestroyed` (entity still alive) and evaluates: (1) `CellDestroyed` trigger on all entities via global sweep, (2) `Death` trigger on the cell's own EffectChains. Then writes `CellDestroyedAt` as aftermath for location-only consumers.

**Arm routing**: When an Arm result's inner trigger belongs to a different entity type, the armed chain moves to that entity's `ArmedEffects`. E.g., `When(PerfectBump, [When(Impact(Wall), ...)])` — first Arm on breaker, second Arm moves to bolt.

**`On` node**: `EffectNode::On { target: Target, then: Vec<EffectNode> }` redirects effect execution to a target entity. For `Do` children, fires the effect targeting that entity. For `When`/`Until`/`Once` children, pushes them onto the target entity's `EffectChains`. Target resolved from trigger context (e.g., `On(Cell)` → cell entity from `BoltHitCell` message).

**Effects are pure data**: No `Target` field on any effect. `Target` enum (`Bolt`, `Breaker`, `Cell`, `Wall`, `AllBolts`, `AllCells`) lives only on `On`. Top-level `On` is enforced at compile time via `RootEffect` single-variant wrapper — `ChipDefinition.effects: Vec<RootEffect>`, `BreakerDefinition.effects: Vec<RootEffect>`.

**`ActiveEffects` (global resource)**: Deleted. All chains are entity-local.
