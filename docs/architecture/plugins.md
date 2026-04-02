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
├── chips/            # Chip system — ChipTemplateRegistry (SeedableRegistry) + ChipCatalog (expanded definitions + recipes); EvolutionTemplateRegistry (SeedableRegistry for evolution definitions); observer-based effect application
├── fx/               # Cross-cutting visual effects (fade-out, node transition overlays)
├── run/              # Run state, node sequencing (node/ sub-domain), timer, RunStats accumulation, HighlightTracker, highlight detection (highlights/ sub-domain), spawn_highlight_text juice
├── audio/            # Event-driven audio, adaptive intensity (stub — Phase 6)
├── ui/               # HUD, menus, chip selection screen
└── debug/            # Dev tooling: overlays, telemetry, hot-reload, recording (sub-domains)
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**Scenario runner exception** — `bolt`, `breaker`, `cells`, `chips`, `effect`, `input`, `run`, `screen`, and `wall` are declared as `pub mod` in `lib.rs` (not `pub(crate)`) because `breaker-scenario-runner` needs cross-crate access to their components, resources, and system sets for entity tagging, input injection, invariant checking, and ordering constraints. `screen` is imported for `ChipOffers` and `ChipOffering` used in chip-selection invariant checks. `wall` and `ui` are also `pub mod` but are not currently imported by the scenario runner. This mirrors the existing debug domain exception.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins. Plugin registration order: `InputPlugin`, `ScreenPlugin`, `RantzSpatial2dPlugin::<GameDrawLayer>`, `RantzPhysics2dPlugin`, `WallPlugin`, `BreakerPlugin`, `EffectPlugin`, `BoltPlugin`, `CellsPlugin`, `ChipsPlugin`, `FxPlugin`, `RunPlugin`, `AudioPlugin`, `UiPlugin`, `DebugPlugin`.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- **Writes** to other domains only through messages — no direct mutation of another domain's components or resources
- **Reads** from other domains' types (components, messages, resources) are normal ECS patterns and not violations — see "Cross-Domain Read Access" below

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `EffectSystems` (`effect/sets.rs`, variants: `Bridge`), `UiSystems` (`ui/sets.rs`), `NodeSystems` (`run/node/sets.rs`). The external crates also export ordering sets: `rantzsoft_physics2d::PhysicsSystems` (`MaintainQuadtree`, `EnforceDistanceConstraints`) for ordering against the quadtree; `rantzsoft_spatial2d::SpatialSystems` (`SavePrevious`, `ApplyVelocity`, `ComputeGlobals`, `DeriveTransform`) for ordering against the spatial pipeline stages; `rantzsoft_defaults::DefaultsSystems` (`Seed`, `PropagateDefaults`) for ordering config-seeding systems via `RantzDefaultsPlugin`. See [ordering.md](ordering.md) for the full table and usage rules.

## Cross-Domain Read Access

The architectural boundary is about **writes** (mutations), not reads. Domains freely **read** other domains' types — components, message types, resources — via standard ECS queries. This is normal Bevy and not a violation:

- **bolt** (collision systems) reads `PiercingRemaining` (bolt domain — bolt gameplay state), `ActivePiercings`, `ActiveDamageBoosts` (effect domain) from bolt entities, `CellHealth` (cells domain) from cell entities, and `BaseWidth`, `BaseHeight` (shared domain) from the breaker entity. The bolt collision systems also write message types owned by other domains (e.g., writing a cells-domain `DamageCell` message). This is expected — collision is a cross-cutting concern now hosted in the bolt domain.
- **cells** receives pre-computed damage via the `DamageCell` message — it does not read `ActiveDamageBoosts` directly. The bolt domain's `bolt_cell_collision` applies the multiplier when writing the message.
- **breaker** reads `ActiveSpeedBoosts`, `ActiveSizeBoosts` (effect domain) from its own entity.
- **effect** reads `BumpPerformed`, `BumpWhiffed` (breaker domain), `BoltImpactCell`, `BoltImpactBreaker`, `BoltImpactWall`, `BreakerImpactCell`, `BreakerImpactWall`, `BoltLost` (bolt/breaker domains), and `RequestCellDestroyed` / `CellDestroyedAt` (cells domain) messages in bridge systems.

**The rule**: any domain may `use crate::other_domain::*` for read-only queries and message consumption. No domain writes to another domain's canonical components or resources directly — that flows through messages. The `debug/` domain is the accepted exception (read AND write, compiled out of release builds). There is one additional narrow production exception — see "ShieldActive Cross-Domain Write" below.

## ShieldActive Cross-Domain Write Exception

`ShieldActive` (effect domain component) is written by two non-effect domains as an accepted architectural exception:

- **bolt** (`bolt_lost` system): reads `ShieldActive` on the **breaker** entity to absorb bolt losses. Decrements `charges` directly; removes `ShieldActive` when charges reach zero. This avoids the round-trip cost of a message for a tight gameplay loop (bolt-lost detection must immediately suppress the `BoltLost` message, not react to it in a subsequent frame).
- **cells** (`handle_cell_hit` system): reads `ShieldActive` on **cell** entities to absorb damage hits. Decrements `charges` directly; removes `ShieldActive` when charges reach zero. Same rationale: damage absorption must short-circuit within the same frame as the hit.

Both systems use `Commands::remove::<ShieldActive>()` to despawn the component when charges are exhausted — the removal is deferred to apply-deferred, not immediate. Neither system fires messages in lieu of writing directly. This pattern is intentional and narrow: `ShieldActive` charge management is co-located with the systems that trigger the absorption.

## Velocity2D Cross-Domain Write Exception

`Velocity2D` (rantzsoft_spatial2d component) on bolt entities is written by effect domain systems as an accepted architectural exception. Three write paths exist:

- **effect** (`apply_gravity_pull` in `effect/effects/gravity_well/effect.rs`): steers bolt velocity toward active gravity wells each FixedUpdate tick. Uses `SpatialData` query and calls `apply_velocity_formula` after steering to enforce speed constraints.
- **effect** (`apply_attraction` in `effect/effects/attraction/effect.rs`): steers bolt velocity toward the nearest attraction target each FixedUpdate tick. Uses `SpatialData` query and calls `apply_velocity_formula` after steering. Ordered `.after(PhysicsSystems::MaintainQuadtree)` for quadtree lookups.
- **effect** (`speed_boost::fire()` / `reverse()` in `effect/effects/speed_boost.rs`): immediately recalculates bolt velocity via `recalculate_velocity` (calls `apply_velocity_formula`) when a speed boost is applied or removed. This ensures bolt speed reflects the new multiplier without waiting for the next tick.

All paths call `apply_velocity_formula` to enforce `(base_speed * boost_mult).clamp(min, max)` magnitude. This is the same velocity enforcement used by collision systems — there is no separate `prepare_bolt_velocity` step.

## Debug Domain — Cross-Domain Exception

The `debug/` domain (gated behind `#[cfg(feature = "dev")]`) is the **only domain permitted to read AND write other domains' resources and components** directly. This is an accepted architectural exception because:

- Hot-reload systems must write to `Res<*Config>` and insert/update entity components across all domains
- Telemetry systems must read components and resources from every domain for display
- Recording systems capture `GameAction` inputs for later scripted playback
- All debug code is compiled out of release builds — it cannot introduce production coupling

This exception does **not** extend to other domains. Production code still communicates through messages only.

## Effect Domain — Self-Registration Pattern

The `effect/` domain uses a self-registration pattern where each leaf effect is fully self-contained in a single file and registers itself with the app.

### Actual Structure

```
effect/
  core/
    mod.rs             # Re-exports from types/
    types/             # Directory module (split from types.rs)
      mod.rs           # Re-exports from definitions/
      definitions/     # Directory module (split from definitions.rs for fire/reverse line count)
        enums.rs       # All core types: Trigger, ImpactTarget, Target, AttractionType,
                       #   RootEffect, EffectNode, EffectKind, BoundEffects, StagedEffects, EffectSourceChip
        fire.rs        # EffectKind::fire() + 3 private helpers
        reverse.rs     # EffectKind::reverse() + 3 private helpers
  effects/             # Per-effect modules with fire(), reverse(), register()
    mod.rs             # pub mod declarations + register() dispatcher
    speed_boost.rs     # ActiveSpeedBoosts, fire(), reverse(), register()
    damage_boost.rs    # fire(), reverse(), register()
    life_lost.rs       # fire(), reverse(), register()
    ramping_damage.rs  # fire(), reverse(), register()
    quick_stop.rs      # fire(), reverse(), register()
    flash_step.rs      # fire(), reverse(), register()
    piercing.rs / size_boost.rs / bump_force.rs / shield.rs / time_penalty.rs
    gravity_well/          # Directory module (split for tests)
    shockwave/ chain_bolt/ chain_lightning/ explode/ tether_beam/ pulse/ piercing_beam/
    attraction/ spawn_bolts/ spawn_phantom/ entropy_engine/ second_wind/ random_effect/
    anchor/ circuit_breaker/ mirror_protocol/
    ... (directory modules — split per System File Split Convention when tests exceed ~400 lines)
  triggers/            # Bridge systems (one file or dir per trigger type)
    mod.rs             # pub mod declarations + register() dispatcher
    evaluate/          # Directory module — shared chain evaluation helpers (has tests)
    impact/            # Directory module — global impact triggers (has tests)
    impacted/          # Directory module — targeted impacted triggers (has tests)
    until/             # Directory module — Until desugaring system (has tests)
    bump.rs / perfect_bump.rs / early_bump.rs / late_bump.rs
    bump_whiff.rs / no_bump.rs
    bumped.rs / perfect_bumped.rs / early_bumped.rs / late_bumped.rs
    bolt_lost.rs / cell_destroyed.rs / death.rs / died.rs
    node_start.rs / node_end.rs / timer.rs
  commands.rs          # EffectCommandsExt trait (fire_effect, reverse_effect, transfer_effect)
  mod.rs               # Re-exports + pub mod declarations
  plugin.rs            # EffectPlugin — calls effects::register() and triggers::register()
  sets.rs              # EffectSystems::Bridge set
```

### Effect File Pattern

Each effect module provides three free functions and any active-state components it needs:

```rust
// effect/effects/speed_boost.rs

// Active state (vec of applied multipliers on the entity)
pub struct ActiveSpeedBoosts(pub Vec<f32>);

impl ActiveSpeedBoosts {
    // Consumers call this inline — no separate cache component
    pub fn multiplier(&self) -> f32 { ... }
}

// Fire: push multiplier onto the vec (inserts component if absent), then recalculate_velocity
pub(crate) fn fire(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) { ... }

// Reverse: remove matching entry from the vec, then recalculate_velocity
pub(crate) fn reverse(entity: Entity, multiplier: f32, _source_chip: &str, world: &mut World) { ... }

// Self-registration: wires app systems (effects with no runtime systems may be empty)
pub(crate) fn register(app: &mut App) { ... }
```

`EffectKind` dispatches to each module via exhaustive match arms. `EffectPlugin::build()` calls `effects::register(app)` and `triggers::register(app)`.

### Effect Dispatch via Commands Extension

Effects are fired through `EffectCommandsExt` on `Commands`:

- `commands.fire_effect(entity, effect, source_chip)` — queues `FireEffectCommand` → calls `effect.fire(entity, &source_chip, world)` at apply
- `commands.reverse_effect(entity, effect, source_chip)` — queues `ReverseEffectCommand` → calls `effect.reverse(entity, &source_chip, world)` at apply
- `commands.transfer_effect(entity, name, children, permanent, context)` — pushes non-Do children to `BoundEffects` (permanent) or `StagedEffects` (one-shot); fires Do children immediately; `context` carries trigger entity references for targeted `On` resolution
- `commands.push_bound_effects(entity, effects)` — inserts `BoundEffects` + `StagedEffects` if absent, then appends pre-built `(String, EffectNode)` entries to `BoundEffects`; used by dispatch systems that bypass the chip-name routing in `transfer_effect`

### Chain Ownership Model

Two effect stores serve different roles:

- **`BoundEffects`** — component on entities (`Vec<(String, EffectNode)>`). Permanent chains that re-evaluate on every matching trigger. Populated at chip dispatch and by `On(permanent: true)` redirects.
- **`StagedEffects`** — component on entities (`Vec<(String, EffectNode)>`). One-shot chains consumed when matched. Populated by `On(permanent: false)` redirects and `Once` wrappers.

**Until desugaring**: `Until` nodes are desugared by a dedicated system. Fires Do children immediately (via `fire_effect`), installs non-Do children into `BoundEffects`, then replaces itself with `When(trigger, [Reverse(effects, chains)])`. When the trigger fires, the Reverse node calls `reverse_effect` for each fired effect and removes chains from `BoundEffects`.
