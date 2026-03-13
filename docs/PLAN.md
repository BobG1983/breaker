# Brickbreaker Roguelite — Build Plan

## Context
Building a roguelite Arkanoid clone in Bevy (Rust). Solo dev, architecture-first approach validated through a vertical slice. Stylized shader-driven visuals, custom physics, hybrid data model (mechanics in code, content in data files), audio planned from the start.

See `DESIGN.md` for core design principles, `architecture/` for technical decisions, and `TERMINOLOGY.md` for game vocabulary.

## Phases

### Done
- [Phase 0: Project Scaffolding & Architecture](plan/done/phase-0-scaffolding.md)
- [Phase 1a: Breaker System](plan/done/phase-1a-breaker.md)
- [Phase 1b: Bolt](plan/done/phase-1b-bolt.md)
- [Phase 1c: Cells](plan/done/phase-1c-cells.md)
- [Phase 1d: Bump & Perfect Bump](plan/done/phase-1d-bump.md)

### Current
- [Phase 2: Game Loop & Time Pressure](plan/phase-2/README.md)
  - [2a: Level Loading & Dev Tooling](plan/phase-2/phase-2a-level-loading.md)
  - [2b: Run Structure & Node Timer](plan/phase-2/phase-2b-run-and-timer.md)
  - [2c: Breaker Archetypes](plan/phase-2/phase-2c-breaker-archetypes.md)
  - [2d: Screens & UI](plan/phase-2/phase-2d-screens-and-ui.md)

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

## Open Questions
1. **Portal cell scope**: Sub-levels are a game-within-a-game. How complex? Defer to Phase 8?
2. **Breaker ability design**: Beyond bolt-lost behavior, what active abilities do breakers have? (Defined per-breaker in data?)
