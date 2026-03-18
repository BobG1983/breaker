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

## Documentation

| Document | Contents |
|----------|----------|
| [Design Principles](docs/design/) | Core design pillars, identity, and design decisions |
| [Architecture](docs/architecture/) | Plugin structure, code standards, message table, patterns |
| [Build Plan](docs/plan/) | Phased roadmap from scaffolding through polish |
| [Terminology](docs/design/terminology.md) | Game vocabulary used in all code and docs |

## Tech Stack

- **Bevy 0.18.1** — ECS game engine
- **Rust 2024 edition** — plugin-per-domain architecture
- **RON data files** — content definitions (cells, upgrades, layouts)

## Project Status

**Phases 0–2b complete, Phase 2c–e in progress.** Core mechanics (breaker, bolt, cells, physics, bump system), level loading, run structure with node timer, main menu, run-end screen, side panel UI with timer HUD and lives display, and the Aegis breaker archetype are all implemented. Next up: remaining Phase 2 work (Chrono & Prism archetypes, screens polish) then Phase 3 dev infrastructure.

## License

Proprietary — all rights reserved. See [LICENSE.md](LICENSE.md).
