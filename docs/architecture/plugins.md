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
├── rantzsoft_defaults/       # Config/defaults pipeline: GameConfig derive macro, RON asset loader, seed/propagate systems, DefaultsSystems set, RantzDefaultsPlugin, SeedableRegistry trait
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs            # Re-exports GameConfig, SeedableConfig; declares all modules
│       ├── handle.rs         # DefaultsHandle<D> resource (typed asset handle wrapper)
│       ├── loader.rs         # RonAssetLoader<T> generic RON AssetLoader + deserialize_ron helper
│       ├── plugin.rs         # RantzDefaultsPlugin, RantzDefaultsPluginBuilder (add_config/add_registry API), DefaultsSystems set (Seed, PropagateDefaults)
│       ├── prelude.rs        # Public re-exports: GameConfig, SeedableConfig, SeedableRegistry, RegistryHandles, DefaultsHandle, RonAssetLoader, DefaultsSystems, RantzDefaultsPlugin, RantzDefaultsPluginBuilder
│       ├── registry.rs       # SeedableRegistry trait (asset_dir, extensions, seed, update_single, update_all); RegistryHandles<A> resource (folder + typed handles)
│       ├── seedable.rs       # SeedableConfig trait (asset_path, extensions, Config associated type)
│       └── systems.rs        # seed_config, propagate_defaults, init_defaults_handle, seed_registry, propagate_registry, init_registry_handles generic systems
├── rantzsoft_defaults_derive/ # Proc-macro crate: #[derive(GameConfig)] for RON defaults loading
│   ├── Cargo.toml
│   └── src/lib.rs
├── rantzsoft_stateflow/      # Game-agnostic state routing, screen transitions, lifecycle messages
│   ├── Cargo.toml            # Package: rantzsoft_stateflow
│   └── src/                  # Route builder, dispatch, transition effects, cleanup
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
├── prelude/          # Cross-domain import convenience — re-exports only, no types
├── shared/           # Shared types: BaseWidth, BaseHeight, PlayfieldConfig, NodeScalingFactor (cleanup markers CleanupOnExit<S> come from rantzsoft_stateflow)
├── state/            # State lifecycle, routing, menus, pause, run/node management, HUD
├── input/            # Raw keyboard input to GameAction translation
├── breaker/          # Breaker mechanics, state machine, bump system
├── effect_v3/        # Effect system — data-driven Tree trigger/effect evaluation and dispatch (top-level domain)
├── bolt/             # Bolt physics, reflection model, speed management, CCD collision detection, chain bolts
├── cells/            # Cell types, grid layout, destruction
├── walls/            # Wall builder, wall types, boundary entities
├── chips/            # Chip system — ChipTemplateRegistry (SeedableRegistry) + ChipCatalog (expanded definitions + recipes); EvolutionTemplateRegistry (SeedableRegistry for evolution definitions); observer-based effect application
├── fx/               # Cross-cutting visual effects (fade-out, node transition overlays)
├── audio/            # Event-driven audio, adaptive intensity (stub — Phase 6)
└── debug/            # Dev tooling: overlays, telemetry, hot-reload, recording (sub-domains)
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**Scenario runner exception** — `bolt`, `breaker`, `cells`, `chips`, `effect_v3`, `input`, `state`, and `walls` are declared as `pub mod` in `lib.rs` (not `pub(crate)`) because `breaker-scenario-runner` needs cross-crate access to their components, resources, and system sets for entity tagging, input injection, invariant checking, and ordering constraints. `state` is imported for `ChipOffers` and `ChipOffering` used in chip-selection invariant checks. This mirrors the existing debug domain exception.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins. Plugin registration order: `InputPlugin`, `StatePlugin`, `RantzSpatial2dPlugin::<GameDrawLayer>`, `RantzPhysics2dPlugin`, `WallPlugin`, `BreakerPlugin`, `EffectV3Plugin`, `BoltPlugin`, `CellsPlugin`, `ChipsPlugin`, `FxPlugin`, `AudioPlugin`, `DebugPlugin`.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- **Writes** to other domains only through messages — no direct mutation of another domain's components or resources
- **Reads** from other domains' types (components, messages, resources) are normal ECS patterns and not violations — see "Cross-Domain Read Access" below

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `EffectV3Systems` (`effect_v3/sets.rs`, variants: `Bridge`, `Tick`, `Conditions`, `Reset`), `UiSystems` (`state/run/node/hud/sets.rs`), `NodeSystems` (`state/run/node/sets.rs`). The external crates also export ordering sets: `rantzsoft_physics2d::PhysicsSystems` (`MaintainQuadtree`, `EnforceDistanceConstraints`) for ordering against the quadtree; `rantzsoft_spatial2d::SpatialSystems` (`SavePrevious`, `ApplyVelocity`, `ComputeGlobals`, `DeriveTransform`) for ordering against the spatial pipeline stages; `rantzsoft_defaults::DefaultsSystems` (`Seed`, `PropagateDefaults`) for ordering config-seeding systems via `RantzDefaultsPlugin`. See [ordering.md](ordering.md) for the full table and usage rules.

## Cross-Domain Read Access

The architectural boundary is about **writes** (mutations), not reads. Domains freely **read** other domains' types — components, message types, resources — via standard ECS queries. This is normal Bevy and not a violation:

- **bolt** (collision systems) reads `PiercingRemaining` (bolt domain — bolt gameplay state), `ActivePiercings`, `ActiveDamageBoosts` (effect domain) from bolt entities, `Hp` (shared death pipeline) from cell entities, and `BaseWidth`, `BaseHeight` (shared domain) from the breaker entity. The bolt collision systems also write message types owned by other domains (e.g., writing `DamageDealt<Cell>` into the unified death pipeline). This is expected — collision is a cross-cutting concern now hosted in the bolt domain.
- **cells** receives pre-computed damage via the `DamageDealt<Cell>` message (unified death pipeline) — it does not read `ActiveDamageBoosts` directly. The bolt domain's `bolt_cell_collision` applies the multiplier when writing the message.
- **breaker** reads `ActiveSpeedBoosts`, `ActiveSizeBoosts` (effect domain) from its own entity.
- **effect** reads `BumpPerformed`, `BumpWhiffed` (breaker domain), `BoltImpactCell`, `BoltImpactBreaker`, `BoltImpactWall`, `BreakerImpactCell`, `BreakerImpactWall`, `BoltLost` (bolt/breaker domains), and `Destroyed<Cell>` / `Destroyed<Bolt>` / `Destroyed<Breaker>` / `Destroyed<Wall>` (unified death pipeline) messages in bridge systems.

**The rule**: any domain may `use crate::other_domain::*` for read-only queries and message consumption. No domain writes to another domain's canonical components or resources directly — that flows through messages. The `debug/` domain is the accepted exception (read AND write, compiled out of release builds).

## Velocity2D Cross-Domain Write Exception

`Velocity2D` (rantzsoft_spatial2d component) on bolt entities is written by effect domain systems as an accepted architectural exception. Three write paths exist:

- **effect** (`apply_gravity_pull` in `effect_v3/effects/gravity_well/effect.rs`): steers bolt velocity toward active gravity wells each FixedUpdate tick. Uses `SpatialData` query and calls `apply_velocity_formula` after steering to enforce speed constraints.
- **effect** (`apply_attraction` in `effect_v3/effects/attraction/effect.rs`): steers bolt velocity toward the nearest attraction target each FixedUpdate tick. Uses `SpatialData` query and calls `apply_velocity_formula` after steering. Ordered `.after(PhysicsSystems::MaintainQuadtree)` for quadtree lookups.
- **effect** (`speed_boost::fire()` / `reverse()` in `effect_v3/effects/speed_boost.rs`): immediately recalculates bolt velocity via `recalculate_velocity` (calls `apply_velocity_formula`) when a speed boost is applied or removed. This ensures bolt speed reflects the new multiplier without waiting for the next tick.

All paths call `apply_velocity_formula` to enforce `(base_speed * boost_mult).clamp(min, max)` magnitude. This is the same velocity enforcement used by collision systems — there is no separate `prepare_bolt_velocity` step.

## Debug Domain — Cross-Domain Exception

The `debug/` domain (gated behind `#[cfg(feature = "dev")]`) is the **only domain permitted to read AND write other domains' resources and components** directly. This is an accepted architectural exception because:

- Hot-reload systems must write to `Res<*Config>` and insert/update entity components across all domains
- Telemetry systems must read components and resources from every domain for display
- Recording systems capture `GameAction` inputs for later scripted playback
- All debug code is compiled out of release builds — it cannot introduce production coupling

This exception does **not** extend to other domains. Production code still communicates through messages only.

## Effect Domain — Trait-Based Dispatch with Per-Effect Configs

The `effect_v3/` domain stores effect trees in `Tree`-typed components (`BoundEffects`, `StagedEffects`), walks them via per-node evaluators in `walking/`, and dispatches each effect through a config struct that implements the `Fireable` (and optionally `Reversible`) trait. There is no `EffectKind` enum holding methods — the enum (`EffectType`) is the dispatch layer and the per-effect config struct is the implementation.

The full architecture (directory layout, type system, walker, conditions, dispatch) lives under `architecture/effects/`. This section captures only the high-level shape relevant to the plugin model.

### Plugin

`EffectV3Plugin` (`effect_v3/plugin.rs`) does four things in `build`:

1. Configures `EffectV3Systems` ordering (`Bridge → Tick → Conditions`).
2. Registers `evaluate_conditions` into `EffectV3Systems::Conditions`.
3. Inserts `SpawnStampRegistry` and registers the four spawn-watcher systems (`stamp_spawned_bolts/cells/walls/breakers`) into `EffectV3Systems::Bridge`.
4. Calls each trigger category's `register::register(app)` (`bump`, `impact`, `death`, `bolt_lost`, `node`, `time`) and each effect config's `Fireable::register(app)` (all 30 configs called unconditionally so adding tick systems later cannot be silently dropped).

### Type system at a glance

- `EffectType` — 30-variant enum, each variant wraps a per-effect `Config` struct (`SpeedBoost(SpeedBoostConfig)`, etc.). Every config implements `Fireable`.
- `ReversibleEffectType` — 16-variant subset for effects whose configs implement `Reversible`. Used in `ScopedTree::Fire` (the only `Fire` position inside `During`/`Until` scopes).
- `Tree` — recursive node enum (`Fire`, `When`, `Once`, `During`, `Until`, `Sequence`, `On`). Stored in `BoundEffects` / `StagedEffects`.
- `RootNode` — top-level entry point (`Stamp(StampTarget, Tree)` or `Spawn(EntityKind, Tree)`). Used by chip/breaker/cell definitions.
- `Trigger` — game events bridged from messages.
- `Condition` — state predicates polled by `evaluate_conditions`.
- `EffectCommandsExt` — extension trait on `Commands` with eight methods: `fire_effect`, `reverse_effect`, `route_effect`, `stamp_effect`, `stage_effect`, `remove_effect`, `remove_staged_effect`, `track_armed_fire`.

### Dispatch layer (free functions, not enum methods)

`fire_dispatch` (`effect_v3/dispatch/fire_dispatch.rs`) is a free function that pattern-matches on `EffectType` and calls the relevant `config.fire(entity, source, world)`:

```rust
pub fn fire_dispatch(effect: &EffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        EffectType::SpeedBoost(config) => config.fire(entity, source, world),
        // ... one arm per variant
    }
}
```

`reverse_dispatch`, `fire_reversible_dispatch`, and `reverse_all_by_source_dispatch` (in `dispatch/reverse_dispatch/system.rs`) handle `ReversibleEffectType` analogously. The walker queues `FireEffectCommand` / `ReverseEffectCommand`; those commands call the dispatch functions inside `Command::apply` where `&mut World` is available.

### Chain storage

Two components, both flat tuple vecs:

- **`BoundEffects(pub Vec<(String, Tree)>)`** — permanent entries that re-arm on every matching trigger. Removed only by `commands.remove_effect`, `Tree::Once` self-removal, or condition disarm.
- **`StagedEffects(pub Vec<(String, Tree)>)`** — one-shot entries consumed when their top-level gate matches. Walker uses entry-specific `(name, tree)` tuple matching to consume the right entry without wiping fresh same-name stages queued during the same evaluation.

There is no internal indexing by trigger/condition/source — chip counts per entity are small enough that linear scan plus structural enum equality is the right trade-off.

### Until and During: state machines, not desugaring

`Tree::Until` is **not** desugared into other node types. `evaluate_until` queues an `UntilEvaluateCommand` that runs a small state machine inside `Command::apply`, using a per-entity `UntilApplied` component to track which sources have already fired. There is no `Reverse` node and no separate desugaring pass.

`Tree::During` is similarly not desugared. `evaluate_during` queues a `DuringInstallCommand` that idempotently inserts the During into `BoundEffects` under a `#installed[0]` key. The `evaluate_conditions` poller (in `EffectV3Systems::Conditions`) iterates installed Durings every tick and fires/reverses scoped trees on transitions tracked by a `DuringActive` component.

Both state machines call `reverse_dispatch` directly from inside their command/system rather than going through `commands.reverse_effect`, because they already hold `&mut World`.

See `architecture/effects/` for the full breakdown:

- `structure.md` — directory layout
- `core_types.md` — type system reference
- `node_types.md` — Tree variants and ScopedTree restrictions
- `evaluation.md` — walker entry points and per-node evaluators
- `until.md` — Until state machine and the four shapes
- `conditions.md` — During poller and the four shapes
- `commands.md` — EffectCommandsExt and concrete commands
- `dispatch.md` — chip dispatch flow and spawn watchers
- `ordering.md` — EffectV3Systems sets and FixedUpdate placement
