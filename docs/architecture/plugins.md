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
├── behaviors/        # Archetype behavior system — trigger→consequence dispatch (top-level domain)
├── bolt/             # Bolt physics, reflection model, speed management
├── cells/            # Cell types, grid layout, destruction
├── wall/             # Invisible boundary entities (left, right, ceiling)
├── chips/            # Amps, Augments, Overclocks system — registry, effect types, observer-based application
├── fx/               # Cross-cutting visual effects (fade-out, flash, particles)
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
- Communicates outward only through messages — no direct cross-module imports for data flow

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `PhysicsSystems` (`physics/sets.rs`), `BehaviorSystems` (`behaviors/sets.rs`), `UiSystems` (`ui/sets.rs`), `NodeSystems` (`run/node/sets.rs`). See [ordering.md](ordering.md) for the full table and usage rules.

## Chip Effect — Justified Cross-Domain Component Reads

Chip effect components (defined in `chips/components.rs`) are stamped onto bolt and breaker entities and read by production systems in other domains. This is an accepted pattern, not a violation:

- **physics** reads `Piercing`, `PiercingRemaining`, and `DamageBoost` from bolt entities in `bolt_cell_collision` — needed for pierce lookahead (comparing effective damage against `CellHealth` to decide whether the bolt passes through or reflects). Physics also reads `CellHealth` from cells domain entities for this same pierce lookahead.
- **cells** reads `DamageBoost` from bolt entities in `handle_cell_hit` — computes `BASE_BOLT_DAMAGE * (1.0 + boost)` damage per hit.
- **breaker** reads `WidthBoost`, `TiltControlBoost`, `BreakerSpeedBoost`, and `BumpForceBoost` from breaker entities — these components are on the same entity the breaker domain already owns.

These are **read-only cross-entity queries** (normal ECS) — the chips domain still owns the components and stamps them; other domains only read. No domain writes to another domain's canonical components. The `debug/` domain exception (read AND write across all domains) is separate and more permissive.

## Debug Domain — Cross-Domain Exception

The `debug/` domain (gated behind `#[cfg(feature = "dev")]`) is the **only domain permitted to read AND write other domains' resources and components** directly. This is an accepted architectural exception because:

- Hot-reload systems must write to `Res<*Config>` and insert/update entity components across all domains
- Telemetry systems must read components and resources from every domain for display
- Recording systems capture `GameAction` inputs for later scripted playback
- All debug code is compiled out of release builds — it cannot introduce production coupling

This exception does **not** extend to other domains. Production code still communicates through messages only.
