# Plugin Architecture

One Bevy plugin per **game domain** (breaker, bolt, cells, etc.) — not one per Bevy system function. Each domain plugin encapsulates all the Bevy systems, components, resources, and messages related to that domain.

Everything lives inside a single `brickbreaker` crate:

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
├── chips/            # Amps, Augments, Overclocks system (stub — Phase 8+)
├── fx/               # Cross-cutting visual effects (fade-out, flash, particles)
├── run/              # Run state, node sequencing (node/ sub-domain), timer
├── physics/          # CCD collision detection, collision response
├── interpolate/      # Transform interpolation for smooth rendering between FixedUpdate ticks
├── audio/            # Event-driven audio, adaptive intensity (stub — Phase 6)
├── ui/               # HUD, menus, upgrade selection screen
└── debug/            # Dev tooling: overlays, telemetry, hot-reload (sub-domains)
assets/               # RON data files, shaders, textures, audio (project root, not inside src/)
```

**`lib.rs`** is the library root. It declares `app`, `game`, and `shared` as `pub mod` (needed by the binary and integration tests). Domain modules are `pub(crate) mod` to enforce plugin boundaries at the Rust visibility level. **`main.rs`** is the binary entry point — it calls `brickbreaker::app::build_app().run()`.

**`App`** (`app.rs`) is responsible for constructing the Bevy `App`, adding `DefaultPlugins`, and adding the `Game` plugin group.

**`Game`** (`game.rs`) is a `PluginGroup` responsible for wiring together all domain plugins in the correct order. This is the single place that knows about all plugins.

**Domain plugins** (breaker, bolt, cells, etc.) are self-contained:
- Each defines its own `Plugin` struct implementing `bevy::app::Plugin`
- Registers its Bevy systems, messages, and states
- Owns its components and resources
- Communicates outward only through messages — no direct cross-module imports for data flow

**Nested sub-domain plugins** — a domain may contain child plugins for cohesive subsets of functionality (e.g., breaker archetypes). The parent plugin adds child plugins via `app.add_plugins()`. `game.rs` only knows about top-level plugins. See [layout.md](layout.md) for the full nesting rules and folder structure.

**Cross-domain SystemSet exports** — domains that expose ordering anchors for other domains define a `pub enum {Domain}Systems` in `sets.rs`. Current exported sets: `BreakerSystems` (`breaker/sets.rs`), `BoltSystems` (`bolt/sets.rs`), `PhysicsSystems` (`physics/sets.rs`), `BehaviorSystems` (`behaviors/sets.rs`), `UiSystems` (`ui/sets.rs`). See [ordering.md](ordering.md) for the full table and usage rules.

## Debug Domain — Cross-Domain Exception

The `debug/` domain (gated behind `#[cfg(feature = "dev")]`) is the **only domain permitted to read AND write other domains' resources and components** directly. This is an accepted architectural exception because:

- Hot-reload systems must write to `Res<*Config>` and insert/update entity components across all domains
- Telemetry systems must read components and resources from every domain for display
- All debug code is compiled out of release builds — it cannot introduce production coupling

This exception does **not** extend to other domains. Production code still communicates through messages only.
