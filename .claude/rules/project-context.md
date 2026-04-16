# Project Context

Shared context for all sub-agents. Read this instead of `CLAUDE.md` (which contains orchestrator-specific rules that don't apply to sub-agents).

## Project

Roguelite Arkanoid clone in **Bevy 0.18** (Rust).

## Workspace

Cargo workspace with crate directories at root:
- `breaker-game/` — main game
- `rantzsoft_spatial2d/` — 2D spatial transform plugin
- `rantzsoft_physics2d/` — 2D physics primitives (quadtree, CCD, CollisionLayers, DistanceConstraint)
- `rantzsoft_defaults/` + `rantzsoft_defaults_derive/` — config/defaults pipeline
- `rantzsoft_stateflow/` — state machine flow, screen transitions, cleanup markers
- `breaker-scenario-runner/` — automated gameplay testing

Game-specific crates use `breaker-<name>` prefix; game-agnostic reusable crates use `rantzsoft_*` prefix (see `rantzsoft-crates.md`).

## Architecture

**Plugin-per-domain** with message-driven decoupling. Each domain plugin (input, breaker, bolt, cells, walls, chips, effect_v3, state, fx, audio, debug) owns its components, resources, and systems. Domains communicate only through Bevy 0.18 messages. See `docs/architecture/` for full details.

## Terminology

All code identifiers MUST use game vocabulary: Breaker (paddle), Bolt (ball), Cell (brick), Node (level), Chip (upgrade), Bump (hit), Flux (meta-currency). No generic terms. See `docs/design/terminology/` for the full glossary.

## Key Docs

- `docs/design/` — design pillars and decisions
- `docs/architecture/` — technical decisions, code standards, testing approach
- `docs/design/terminology/` — game vocabulary (required reading for any agent touching game code)
