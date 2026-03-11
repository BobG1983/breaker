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
| [Design Principles](docs/DESIGN.md) | Core design pillars, identity, and non-negotiable rules |
| [Architecture](docs/ARCHITECTURE.md) | Plugin structure, code standards, message table, patterns |
| [Build Plan](docs/PLAN.md) | Phased roadmap from scaffolding through polish |
| [Terminology](docs/TERMINOLOGY.md) | Game vocabulary used in all code and docs |

## Tech Stack

- **Bevy 0.18.1** — ECS game engine
- **Rust 2024 edition** — plugin-per-domain architecture
- **RON data files** — content definitions (cells, upgrades, layouts)

## Project Status

**Phases 0-1 complete, Phase 2 next.** Scaffolding, plugin architecture, and core mechanics (breaker, bolt, cells, physics, bump system) are implemented. Next up: game loop (timer, bolt-lost penalties, level completion, level loading).

## License

Proprietary — all rights reserved. See [LICENSE.md](LICENSE.md).
