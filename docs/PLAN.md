# Brickbreaker Roguelite — Build Plan

## Context
Building a roguelite Arkanoid clone in Bevy (Rust). Solo dev, architecture-first approach validated through a vertical slice. Stylized shader-driven visuals, custom physics, hybrid data model (mechanics in code, content in data files), audio planned from the start.

See `DESIGN.md` for core design principles, `architecture/` for technical decisions, and `TERMINOLOGY.md` for game vocabulary.

## Phases

### Done
- [Phase 0: Project Scaffolding & Architecture](plan/done/phase-0-scaffolding.md)
- [Phase 1: Core Mechanics](plan/done/phase-1/README.md)
  - [1a: Breaker System](plan/done/phase-1/phase-1a-breaker.md)
  - [1b: Bolt](plan/done/phase-1/phase-1b-bolt.md)
  - [1c: Cells](plan/done/phase-1/phase-1c-cells.md)
  - [1d: Bump & Perfect Bump](plan/done/phase-1/phase-1d-bump.md)
- Phase 2 (partial):
  - [2a: Level Loading](plan/done/phase-2/phase-2a-level-loading.md)
  - [2b: Run Structure & Node Timer](plan/done/phase-2/phase-2b-run-and-timer.md)

### Current
- [Phase 2: Game Loop & Time Pressure](plan/phase-2/README.md)
  - [2c: Archetype System & Aegis](plan/phase-2/phase-2c-archetype-system.md)
  - [2d: Screens](plan/phase-2/phase-2d-screens-and-ui.md)
  - [2e: Visual Polish & Additional Archetypes](plan/phase-2/phase-2e-chrono-and-prism.md)

### Upcoming
- [Phase 3: Dev Infrastructure](plan/phase-3/README.md)
  - [3a: Workspace Restructure](plan/phase-3/phase-3a-workspace-restructure.md)
  - [3b: Debug Domain Restructure](plan/phase-3/phase-3b-debug-restructure.md)
  - [3c: RON Hot-Reload](plan/phase-3/phase-3c-hot-reload.md)
  - [3d: Scenario Runner](plan/phase-3/phase-3d-scenario-runner.md)
- [Phase 4: Vertical Slice — Mini-Run](plan/phase-4-vertical-slice.md)
- [Phase 5: Visual Identity](plan/phase-5-visual-identity.md)
- [Phase 6: Audio Foundation](plan/phase-6-audio.md)
- [Phase 7: Content & Variety](plan/phase-7-content.md)
- [Phase 8: Roguelite Progression](plan/phase-8-roguelite.md)
- [Phase 9: Boss Nodes & Advanced Mechanics](plan/phase-9-bosses.md)
- [Phase 10: Polish & Ship](plan/phase-10-polish.md)

## Build Order Rationale

The plan front-loads **feel** (Phases 1-2) because if the breaker-bolt-cell interaction isn't satisfying, nothing else matters. **Dev infrastructure** (Phase 3) comes next — hot-reload and scenario testing make the vertical slice faster to iterate on. The **vertical slice** (Phase 4) validates architecture under real gameplay conditions before investing in content. Visuals and audio (Phases 5-6) come before content expansion because the stylized aesthetic is part of the game's identity and informs how content is designed. Roguelite systems (Phase 8) are deliberately late — they're important for retention but meaningless without solid core gameplay.

## Resolved Design Decisions
- **Upgrade stacking**: Per-effect caps (set high). No literal uncapped multiplicative stacking.
- **Bump multipliers**: All grades boost — early/late = small, perfect = large, no bump = neutral.
- **Upgrade timeout**: Timer expires = skip (no upgrade). Maximum pressure.
- **Seeded determinism**: Phase 4 (vertical slice). Accept retrofit cost; seeds meaningless with 3 layouts.
- **Breaker archetypes**: Proof-of-concept for the system, not final designs. Broader differentiation (base stats + abilities). Upgrade affinities deferred to Phase 7+.
- **Workspace structure**: Axum-style — game/, derive/, scenario-runner/ as peer directories at root.
- **Scenario runner**: Separate crate with argh CLI. Visual + headless modes. RON scenario files crate-local.

## Open Questions
1. **Portal cell scope**: Sub-levels are a game-within-a-game. How complex? Defer to Phase 9?
