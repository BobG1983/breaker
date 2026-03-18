# Brickbreaker Roguelite — Build Plan

## Context
Building a roguelite Arkanoid clone in Bevy (Rust). Solo dev, architecture-first approach validated through a vertical slice. Stylized shader-driven visuals, custom physics, hybrid data model (mechanics in code, content in data files), audio planned from the start.

See `../design/` for core design principles and decisions, `../architecture/` for technical decisions, and `../design/terminology.md` for game vocabulary.

## Phases

### Done
- [Phase 0: Project Scaffolding & Architecture](done/phase-0-scaffolding.md)
- [Phase 1: Core Mechanics](done/phase-1/index.md)
  - [1a: Breaker System](done/phase-1/phase-1a-breaker.md)
  - [1b: Bolt](done/phase-1/phase-1b-bolt.md)
  - [1c: Cells](done/phase-1/phase-1c-cells.md)
  - [1d: Bump & Perfect Bump](done/phase-1/phase-1d-bump.md)
- [Phase 2: Game Loop & Time Pressure](phase-2/index.md)
  - [2a: Level Loading](done/phase-2/phase-2a-level-loading.md)
  - [2b: Run Structure & Node Timer](done/phase-2/phase-2b-run-and-timer.md)
  - [2c: Archetype System & Aegis](done/phase-2/phase-2c-archetype-system.md)
  - [2d: Screens](done/phase-2/phase-2d-screens-and-ui.md)
  - [2e: Visual Polish & Additional Archetypes](done/phase-2/phase-2e-chrono-and-prism.md)
- [Phase 3: Dev Infrastructure](done/phase-3/index.md)
  - [3a: Workspace Restructure](done/phase-3/phase-3a-workspace-restructure.md)
  - [3b: Debug Domain Restructure](done/phase-3/phase-3b-debug-restructure.md)
  - [3c: RON Hot-Reload](done/phase-3/phase-3c-hot-reload.md)
  - [3d: Scenario Runner](done/phase-3/phase-3d-scenario-runner.md)
  - [3e: Structured Logging](done/phase-3/phase-3e-structured-logging.md)

### Current
- [Phase 4: Vertical Slice — Mini-Run](phase-4/index.md) — 4 waves, 8 sessions
  - **Wave 1 — Foundations** (Sessions 1-2)
    - [4a: Seeded RNG & Run Seed](phase-4/phase-4a-seeded-rng.md)
    - [4b: Chip Effect System](phase-4/phase-4b-chip-effects.md) (4b.1 types/stacking, 4b.2 per-domain effects)
  - **Wave 2 — Core Systems** (Sessions 3-6)
    - [4c: Chip Pool & Rarity](phase-4/phase-4c-chip-pool.md) (4c.1 rarity/inventory, 4c.2 RON authoring, 4c.3 synergy review)
    - [4d: Trigger/Effect Architecture](phase-4/phase-4d-trigger-effect.md) (4d.1 types, 4d.2 bolt behaviors, 4d.3 shockwave, 4d.4 Surge POC)
    - [4e: Node Sequence & Escalation](phase-4/phase-4e-node-escalation.md) (4e.1 tiers, 4e.2 proc-gen, 4e.3 cell types, 4e.4 layout pools)
  - **Wave 3 — Integration** (Session 7)
    - [4f: Chip Offering System](phase-4/phase-4f-chip-offerings.md)
    - [4g: Node Transitions & VFX](phase-4/phase-4g-node-transitions.md)
  - **Wave 4 — Capstones** (Session 8)
    - [4h: Chip Evolution](phase-4/phase-4h-chip-evolution.md)
    - [4i: Run Stats & Summary](phase-4/phase-4i-run-stats.md)
    - [4j: Release Infrastructure](phase-4/phase-4j-release-infrastructure.md)

### Upcoming
- [Phase 5: Visual Identity](phase-5-visual-identity.md)
- [Phase 6: Audio Foundation](phase-6-audio.md)
- [Phase 7: Content & Variety](phase-7-content.md)
- [Phase 8: Roguelite Progression](phase-8-roguelite.md)
- [Phase 9: Boss Nodes & Advanced Mechanics](phase-9-bosses.md)
- [Phase 10: Polish & Ship](phase-10-polish.md)

## Build Order Rationale

The plan front-loads **feel** (Phases 1-2) because if the breaker-bolt-cell interaction isn't satisfying, nothing else matters. **Dev infrastructure** (Phase 3) comes next — hot-reload and scenario testing make the vertical slice faster to iterate on. The **vertical slice** (Phase 4) validates architecture under real gameplay conditions before investing in content. Visuals and audio (Phases 5-6) come before content expansion because the stylized aesthetic is part of the game's identity and informs how content is designed. Roguelite systems (Phase 8) are deliberately late — they're important for retention but meaningless without solid core gameplay.

## Resolved Design Decisions

Documented in full at `../design/decisions/`. Summary:

| Decision | Summary | Detail |
|----------|---------|--------|
| Chip stacking | Hybrid: per-chip caps, flat integer stacks, Isaac-style pool depletion | [chip-stacking.md](../design/decisions/chip-stacking.md) |
| Chip evolution | Two chips at min stacks + boss kill = evolved chip | [chip-evolution.md](../design/decisions/chip-evolution.md) |
| Chip offerings | Weighted random, rarity tiers, weight decay on seen chips | [chip-offering-system.md](../design/decisions/chip-offering-system.md) |
| Chip timeout | Timer expires = skip (no chip). Maximum pressure | [chip-timeout.md](../design/decisions/chip-timeout.md) |
| Bump multipliers | All grades boost — early/late = small, perfect = large, no bump = neutral | [bump-multipliers.md](../design/decisions/bump-multipliers.md) |
| Node escalation | Procedural tiers from seed, passive -> active -> boss cadence | [node-escalation.md](../design/decisions/node-escalation.md) |
| Seeded determinism | Phase 4. Run seed drives all randomness | [seeded-determinism.md](../design/decisions/seeded-determinism.md) |
| Chip synergies | 30%+ chips must interact with other chips' effects. Web, not list | [chip-synergies.md](../design/decisions/chip-synergies.md) |
| Breaker archetypes | Proof-of-concept designs, system flexibility over final balance | [breaker-archetypes.md](../design/decisions/breaker-archetypes.md) |
| Workspace structure | Axum-style — `breaker-game/`, `breaker-derive/`, `breaker-scenario-runner/` as peer directories |
| Scenario runner | Separate crate with argh CLI. Visual + headless modes. RON scenario files crate-local |

## Open Questions
1. **Portal cell scope**: Sub-levels are a game-within-a-game. How complex? Defer to Phase 9?
