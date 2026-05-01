# Brickbreaker

A roguelite Arkanoid clone — the reflex pressure of Ikaruga meets the build-crafting depth of Slay the Spire, in a breakout game that never lets you breathe.

## Setup

After cloning:

```bash
# 1. Enable pre-commit hooks (fmt, clippy, tests)
git config --local core.hooksPath .githooks

# 2. Initialize git-flow-next
git flow init --preset=classic --defaults
git flow config add topic refactor develop --prefix=refactor/
git flow config edit topic bugfix --prefix=fix/

# 3. Set merge strategy (preserve branch topology)
git config --local merge.ff false
```

**Prerequisites:**
- Rust toolchain (nightly)
- [mold](https://github.com/rui314/mold) or lld recommended for fast linking

## Build & Run

```
cargo dev                    # Dev build + run (dynamic linking)
cargo dtest                  # Run game crate tests (dynamic linking)
cargo all-dtest              # Run all workspace tests (dynamic linking)
cargo dcheck                 # Type check (dynamic linking)
cargo dclippy                # Lint game crate (dynamic linking)
cargo all-dclippy            # Lint all workspace crates (dynamic linking)
cargo scenario               # Run scenario tests (release build)
cargo run --release          # Release build
cargo fmt --check            # Format check
```

Dev aliases are defined in `.cargo/config.toml` and use `bevy/dynamic_linking` for fast compiles.

## Development Tools

Optional cargo subcommands used by CI and development agents:

```
cargo install cargo-audit     # Dependency vulnerability scanning
cargo install cargo-deny      # License and advisory checks
cargo install cargo-machete   # Unused dependency detection
cargo install cargo-outdated  # Outdated dependency reporting
```

## Documentation

| Document | Contents |
|----------|----------|
| [Design Principles](docs/design/) | Core design pillars, identity, and design decisions |
| [Architecture](docs/architecture/) | Plugin structure, code standards, message table, patterns |
| [Terminology](docs/design/terminology/) | Game vocabulary used in all code and docs |
| [Todo List](docs/todos/TODO.md) | Current backlog and roadmap |

## Tech Stack

- **Bevy 0.18** — ECS game engine
- **Rust 2024 edition (nightly)** — plugin-per-domain architecture, message-driven decoupling
- **RON data files** — all content (chips, evolutions, cells, nodes, breakers, config) is data-driven
- **Cargo workspace** — `breaker-game`, `breaker-scenario-runner`, plus the `rantzsoft_*` crates below

**Reusable crates (game-agnostic):**

| Crate | Purpose |
|-------|---------|
| `rantzsoft_spatial2d` | Position2D, Velocity2D, interpolation, propagation |
| `rantzsoft_physics2d` | CCD, quadtree, collision layers, distance constraints |
| `rantzsoft_dmg` | Damage pipeline — preview, application, armor, pierce |
| `rantzsoft_stateflow` | State routing, screen transitions, cleanup markers |
| `rantzsoft_defaults` | Config/defaults pipeline with derive macro |

## What's Built

Full run loop: main menu → node sequence → chip select → run-end screen.

**Core mechanics:**
- Breaker movement with dash, tilt, and three archetypes (Aegis, Chrono, Prism)
- Bolt physics with continuous collision detection (CCD)
- Bump grading — perfect / early / late / whiff — driving all protocol and chip triggers
- Node timer with bolt-loss penalties
- Six node layouts; four cell types (standard, tough, lock, regen with orbiting shields)
- Toughness/HP scaling with exponential tier progression

**Build system:**
- Chip system — TriggerChain-based effects, all data-driven via RON templates with per-rarity variants, pool depletion, and weight decay
- Evolution system — 8 recipes that combine maxed chips into ultimate abilities
- TriggerChain engine — nested trigger→effect chains (13 leaf effects, 8 trigger types)
- Protocols — 9 run modifiers that rewire how the bolt and breaker behave (Burnout, Fission, Echo Strike, Iron Curtain, Debt Collector, Overcharge, Haste, Cascade, and more)

**Polish:**
- Spreading shockwaves with quadtree spatial queries
- Chain bolts — tethered pairs via distance constraints with momentum conservation
- Shield system — temporary bolt-loss protection with timed expiry
- Highlight popups with punch-scale animation; diversity-penalized run-end scoring

**Not yet built:** graphics/VFX (placeholder), audio, save/load, content expansion, roguelite meta-progression.

## License

Proprietary — all rights reserved. See [LICENSE.md](LICENSE.md).
