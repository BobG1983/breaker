# Brickbreaker

A roguelite Arkanoid clone — the reflex pressure of Ikaruga meets the build-crafting depth of Slay the Spire, in a breakout game that never lets you breathe.

## Build & Run

**Prerequisites:** Rust toolchain (nightly). [mold](https://github.com/rui314/mold) or lld recommended for fast linking.

```
cargo dev                    # Dev build + run (dynamic linking)
cargo dtest                  # Run all tests (dynamic linking)
cargo dcheck                 # Type check (dynamic linking)
cargo dclippy                # Lint (dynamic linking)
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
| [Build Plan](docs/plan/) | Phased roadmap from scaffolding through polish |
| [Terminology](docs/design/terminology.md) | Game vocabulary used in all code and docs |

## Tech Stack

- **Bevy 0.18** — ECS game engine
- **Rust 2024 edition** — plugin-per-domain architecture, message-driven decoupling
- **RON data files** — all content (chips, evolutions, cells, nodes, breakers, config) is data-driven
- **Cargo workspace** — `breaker-game`, `rantzsoft_spatial2d`, `rantzsoft_physics2d`, `rantzsoft_defaults`/`_derive`, `breaker-scenario-runner`

## Project Status

**Phases 0–3 complete. Phase 4 (vertical slice) in progress.**

Core gameplay is fully playable: breaker movement with dash and tilt, bolt physics with CCD collision, bump grading (perfect/early/late/whiff), node timer with penalties, three breaker archetypes (Aegis, Chrono, Prism), six node layouts, four cell types (standard, tough, lock, regen with orbiting shields), and a full run loop (main menu → node sequence → chip select → run-end screen).

### Phase 4 Highlights

- **Chip system**: Amps (bolt upgrades), Augments (breaker upgrades), Overclocks (triggered abilities) — all data-driven via RON, with pool depletion and weight decay for build-crafting depth
- **Evolution system**: 8 evolution recipes that combine maxed chips into ultimate abilities (Nova Lance, Voltchain, Phantom Breaker, Supernova, Dead Man's Hand, Railgun, Gravity Well, Second Wind)
- **TriggerChain engine**: Nested trigger→effect chains with multi-step arming (e.g., OnPerfectBump → OnImpact(Cell) → Shockwave). 13 leaf effects, 8 trigger types
- **Memorable moments**: In-game highlight popups with punch-scale animation, diversity-penalized scoring for run-end display
- **Spreading shockwaves**: Expanding wavefront area damage with quadtree spatial queries
- **Chain bolts**: Tethered bolt pairs via distance constraints with momentum conservation
- **Shield system**: Temporary bolt-loss protection with timed expiry
- **Spatial extraction**: `rantzsoft_spatial2d` (Position2D, Velocity2D, interpolation, propagation) and `rantzsoft_physics2d` (CCD, quadtree, collision layers, distance constraints) as reusable game-agnostic crates

## License

Proprietary — all rights reserved. See [LICENSE.md](LICENSE.md).
