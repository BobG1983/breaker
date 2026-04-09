# Architecture

Technical decisions for how the game is built. See `../design/` for *why* (game feel), `../plan/` for *when* (build phases), `../design/terminology/` for vocabulary.

## Engine & Stack

- **Bevy 0.18.1** — 2D only (`default-features = false, features = ["2d", "serialize"]`)
- **Custom physics** — No rapier. Breakout physics are specialized (angle overwrite, no perfect verticals). Full control needed.
- **Data format** — Hybrid: type-safe mechanics in Rust, tweakable content in RON files
- **Debug UI** — `bevy_egui` for in-game debug console (added Phase 0)

## Sub-Documents

| Document | Contents |
|----------|----------|
| [plugins.md](plugins.md) | Plugin architecture, crate structure, domain plugin rules |
| [layout.md](layout.md) | Canonical domain folder structure and per-file rules |
| [messages.md](messages.md) | Inter-domain communication via Bevy messages |
| [ordering.md](ordering.md) | System ordering, SystemSet conventions, ordering chain |
| [state.md](state.md) | GameState, PlayingState, sub-states, run_if gating |
| [physics.md](physics.md) | FixedUpdate physics, CCD collision, bolt reflection model |
| [content.md](content.md) | Content identity (enum behaviors + RON instances), upgrade application, RON validation |
| [data.md](data.md) | Data model — components vs resources, config pipeline, ownership rules, dimension conventions |
| [standards.md](standards.md) | Code standards, prelude conventions, error handling, testing (TDD), entity cleanup, asset loading, debug console |
| [testing.md](testing.md) | Test infrastructure — composable app builders, domain test_utils, shared helpers, migration plan |
| [effects/](effects/index.md) | Effect system architecture — triggers, commands, dispatch, evaluation, reversal |
