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
├── breaker-derive/           # Proc-macro crate
│   ├── Cargo.toml            # Package: brickbreaker_derive
│   └── src/lib.rs            # GameConfig derive macro
├── breaker-scenario-runner/  # Automated gameplay testing tool (dev-only binary)
│   ├── Cargo.toml            # Package: breaker_scenario_runner
│   ├── src/                  # Runner source (types, lifecycle, invariants, input, log_capture)
│   └── scenarios/            # RON scenario files (crate-local, never shipped)
└── docs/                     # Design docs, architecture, build plan
```

**Naming convention:** Root-level crate directories are named `breaker-<name>`. Cargo package names use underscores (`brickbreaker`, `brickbreaker_derive`, `breaker_scenario_runner`). New crates follow this pattern.

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
├── bolt/             # Bolt physics, reflection model, speed management
├── cells/            # Cell types, grid layout, destruction
├── wall/             # Invisible boundary entities (left, right, ceiling)
├── chips/            # Amps, Augments, Overclocks system — registry, effect types, observer-based application
├── fx/               # Cross-cutting visual effects (fade-out, node transition overlays)
├── run/              # Run state, node sequencing (node/ sub-domain), timer
├── physics/          # CCD collision detection, collision response
├── interpolate/      # Transform interpolation for smooth rendering between FixedUpdate ticks
├── audio/            # Event-driven audio, adaptive intensity (stub — Phase 6)
├── ui/               # HUD, menus, chip selection screen
└── debug/            # Dev tooling: overlays, telemetry, hot-reload, recording (sub-domains)
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**Scenario runner exception** — `bolt`, `breaker`, `input`, `physics`, and `run` are declared as `pub mod` in `lib.rs` (not `pub(crate)`) because `breaker-scenario-runner` needs cross-crate access to their components, resources, and system sets for entity tagging, input injection, invariant checking, and ordering constraints. This mirrors the existing debug domain exception.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- **Writes** to other domains only through messages — no direct mutation of another domain's components or resources
- **Reads** from other domains' types (components, messages, resources) are normal ECS patterns and not violations — see "Cross-Domain Read Access" below

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `PhysicsSystems` (`physics/sets.rs`), `BehaviorSystems` (`behaviors/sets.rs`), `UiSystems` (`ui/sets.rs`), `NodeSystems` (`run/node/sets.rs`). See [ordering.md](ordering.md) for the full table and usage rules.

## Cross-Domain Read Access

The architectural boundary is about **writes** (mutations), not reads. Domains freely **read** other domains' types — components, message types, resources — via standard ECS queries. This is normal Bevy and not a violation:

- **physics** reads `Piercing`, `PiercingRemaining`, `DamageBoost` (chips domain) from bolt entities, `CellHealth`, `CellWidth`, `CellHeight` (cells domain) from cell entities, and `BreakerWidth`, `BreakerHeight` (breaker domain) from the breaker entity. Physics also imports and writes message types owned by other domains (e.g., writing a cells-domain `DamageCell` message). This is expected — physics is a cross-cutting collision service.
- **cells** reads `DamageBoost` (chips domain) from bolt entities in `handle_cell_hit`.
- **breaker** reads `WidthBoost`, `TiltControlBoost`, `BreakerSpeedBoost`, `BumpForceBoost` (chips domain) from its own entity.
- **behaviors** reads `BumpPerformed`, `BumpWhiffed` (breaker domain), `BoltHitCell`, `BoltHitBreaker`, `BoltHitWall`, `BoltLost` (physics domain), and `CellDestroyed` (cells domain) messages in bridge systems.

**The rule**: any domain may `use crate::other_domain::*` for read-only queries and message consumption. No domain writes to another domain's canonical components or resources directly — that flows through messages. The `debug/` domain is the sole exception (read AND write, compiled out of release builds).

## Debug Domain — Cross-Domain Exception

The `debug/` domain (gated behind `#[cfg(feature = "dev")]`) is the **only domain permitted to read AND write other domains' resources and components** directly. This is an accepted architectural exception because:

- Hot-reload systems must write to `Res<*Config>` and insert/update entity components across all domains
- Telemetry systems must read components and resources from every domain for display
- Recording systems capture `GameAction` inputs for later scripted playback
- All debug code is compiled out of release builds — it cannot introduce production coupling

This exception does **not** extend to other domains. Production code still communicates through messages only.
