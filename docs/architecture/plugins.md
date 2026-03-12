# Plugin Architecture

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
├── input/            # Raw keyboard input to GameAction translation
├── breaker/          # Breaker mechanics, state machine, bump system
├── bolt/             # Bolt physics, reflection model, speed management
├── cells/            # Cell types, grid layout, destruction
├── upgrades/         # Amps, Augments, Overclocks system
├── run/              # Run state, seeded node sequencing, timer, difficulty scaling
├── physics/          # CCD collision detection, collision response, wall entities
├── audio/            # Event-driven audio, adaptive intensity
├── ui/               # HUD, menus, upgrade selection screen
└── debug/            # bevy_egui debug console, overlays
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

See [layout.md](layout.md) for the canonical domain folder structure and per-file rules.
