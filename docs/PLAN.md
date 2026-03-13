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

### Current
- [Phase 2: Game Loop & Time Pressure](plan/phase-2/README.md)
  - [2a: Level Loading](plan/phase-2/phase-2a-level-loading.md)
  - [2b: Run Structure & Node Timer](plan/phase-2/phase-2b-run-and-timer.md)
  - [2c: Archetype System & Aegis](plan/phase-2/phase-2c-archetype-system.md)
  - [2d: Screens & UI](plan/phase-2/phase-2d-screens-and-ui.md)
  - [2e: Chrono & Prism](plan/phase-2/phase-2e-chrono-and-prism.md)
  - [2f: Dev Tooling](plan/phase-2/phase-2f-dev-tooling.md)

### Upcoming
- [Phase 3: Vertical Slice — Mini-Run](plan/phase-3-vertical-slice.md)
- [Phase 4: Visual Identity](plan/phase-4-visual-identity.md)
- [Phase 5: Audio Foundation](plan/phase-5-audio.md)
- [Phase 6: Content & Variety](plan/phase-6-content.md)
- [Phase 7: Roguelite Progression](plan/phase-7-roguelite.md)
- [Phase 8: Boss Nodes & Advanced Mechanics](plan/phase-8-bosses.md)
- [Phase 9: Polish & Ship](plan/phase-9-polish.md)

## Build Order Rationale

The plan front-loads **feel** (Phases 1-2) because if the breaker-bolt-cell interaction isn't satisfying, nothing else matters. The vertical slice (Phase 3) validates architecture under real gameplay conditions before investing in content. Visuals and audio (Phases 4-5) come before content expansion because the stylized aesthetic is part of the game's identity and informs how content is designed. Roguelite systems (Phase 7) are deliberately late — they're important for retention but meaningless without solid core gameplay.

## Resolved Design Decisions
- **Upgrade stacking**: Per-effect caps (set high). No literal uncapped multiplicative stacking.
- **Bump multipliers**: All grades boost — early/late = small, perfect = large, no bump = neutral.
- **Upgrade timeout**: Timer expires = skip (no upgrade). Maximum pressure.
- **Seeded determinism**: Phase 3. Accept retrofit cost; seeds meaningless with 3 layouts.
- **Breaker archetypes**: Proof-of-concept for the system, not final designs. Broader differentiation (base stats + abilities). Upgrade affinities deferred to Phase 6+.

## Open Questions
1. **Portal cell scope**: Sub-levels are a game-within-a-game. How complex? Defer to Phase 8?
